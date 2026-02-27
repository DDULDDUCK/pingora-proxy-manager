use super::trusted_proxy;
use super::{FilterResult, ProxyCtx, ProxyFilter};
use crate::auth;
use crate::constants;
use crate::state::AppState;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use std::sync::Arc;

pub struct AclFilter {
    pub state: Arc<AppState>,
}

#[async_trait]
impl ProxyFilter for AclFilter {
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut ProxyCtx,
    ) -> Result<FilterResult> {
        if let Some(host_config) = &ctx.host_config {
            if let Some(acl_id) = host_config.access_list_id {
                if let Some(acl) = self.state.get_access_list(acl_id) {
                    // (A) IP 기반 필터링
                    if !acl.ips.is_empty() {
                        let client_ip = trusted_proxy::effective_client_ip(session)
                            .map(|ip| ip.to_string())
                            .unwrap_or_default();

                        let mut ip_allowed = true;
                        let mut has_allow_rules = false;

                        for rule in &acl.ips {
                            if rule.action == "allow" {
                                has_allow_rules = true;
                                if rule.ip == client_ip {
                                    ip_allowed = true;
                                    break;
                                } else {
                                    ip_allowed = false;
                                }
                            } else if rule.action == "deny" {
                                if rule.ip == client_ip {
                                    tracing::warn!(
                                        "⛔ Access Denied (IP Blocked): {} -> {}",
                                        client_ip,
                                        ctx.host
                                    );
                                    let _ = session.respond_error(constants::http::FORBIDDEN).await;
                                    return Ok(FilterResult::Handled);
                                }
                            }
                        }

                        if has_allow_rules && !ip_allowed {
                            tracing::warn!(
                                "⛔ Access Denied (IP Not Allowed): {} -> {}",
                                client_ip,
                                ctx.host
                            );
                            let _ = session.respond_error(constants::http::FORBIDDEN).await;
                            return Ok(FilterResult::Handled);
                        }
                    }

                    // (B) Basic Auth
                    if !acl.clients.is_empty() {
                        let auth_header = session.req_header().headers.get("Authorization");
                        let mut authenticated = false;

                        if let Some(value) = auth_header {
                            if let Ok(v_str) = value.to_str() {
                                if v_str.starts_with("Basic ") {
                                    let encoded = &v_str[6..];
                                    if let Ok(decoded) = general_purpose::STANDARD.decode(encoded) {
                                        if let Ok(creds) = String::from_utf8(decoded) {
                                            if let Some((username, password)) =
                                                creds.split_once(':')
                                            {
                                                if let Some(client_conf) = acl
                                                    .clients
                                                    .iter()
                                                    .find(|c| c.username == username)
                                                {
                                                    if auth::verify_password(
                                                        password,
                                                        &client_conf.password_hash,
                                                    ) {
                                                        authenticated = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if !authenticated {
                            tracing::info!("🔒 Authentication required for {}", ctx.host);
                            let mut header =
                                match ResponseHeader::build(constants::http::UNAUTHORIZED, Some(4))
                                {
                                    Ok(h) => h,
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to build 401 response header: {}",
                                            e
                                        );
                                        let _ = session
                                            .respond_error(constants::http::INTERNAL_ERROR)
                                            .await;
                                        return Ok(FilterResult::Handled);
                                    }
                                };
                            if let Err(e) = header.insert_header(
                                "WWW-Authenticate",
                                "Basic realm=\"Restricted Area\"",
                            ) {
                                tracing::error!("Failed to insert WWW-Authenticate header: {}", e);
                                let _ =
                                    session.respond_error(constants::http::INTERNAL_ERROR).await;
                                return Ok(FilterResult::Handled);
                            }
                            if let Err(e) = header.insert_header("Content-Length", "0") {
                                tracing::error!("Failed to insert Content-Length header: {}", e);
                                let _ =
                                    session.respond_error(constants::http::INTERNAL_ERROR).await;
                                return Ok(FilterResult::Handled);
                            }
                            session
                                .write_response_header(Box::new(header), true)
                                .await?;
                            return Ok(FilterResult::Handled);
                        }
                    }
                }
            }
        }
        Ok(FilterResult::Continue)
    }
}
