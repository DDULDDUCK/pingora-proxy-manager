use super::trusted_proxy;
use super::{FilterResult, ProxyCtx, ProxyFilter};
use crate::constants;
use async_trait::async_trait;
use pingora::http::ResponseHeader;
use pingora::prelude::*;

pub struct SslFilter;

fn configured_tls_port() -> Option<u16> {
    constants::network::TLS_PORT_STR
        .rsplit(':')
        .next()
        .and_then(|p| p.parse::<u16>().ok())
}

#[async_trait]
impl ProxyFilter for SslFilter {
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut ProxyCtx,
    ) -> Result<FilterResult> {
        if let Some(host_config) = &ctx.host_config {
            let forwarded_proto_is_https = trusted_proxy::forwarded_proto_is_https(session);

            let is_tls = if forwarded_proto_is_https {
                true
            } else {
                let tls_port = configured_tls_port();
                session
                    .server_addr()
                    .map(|a| {
                        a.as_inet()
                            .map(|s| tls_port.map(|p| s.port() == p).unwrap_or(false))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            };

            if host_config.ssl_forced && !is_tls {
                let path = session.req_header().uri.path();
                let query = session
                    .req_header()
                    .uri
                    .query()
                    .map(|q| format!("?{}", q))
                    .unwrap_or_default();
                let new_url = format!("https://{}{}{}", ctx.host, path, query);

                let mut header = ResponseHeader::build(301, Some(4)).map_err(|e| {
                    Error::explain(
                        ErrorType::InternalError,
                        format!("Failed to build 301 header: {}", e),
                    )
                })?;
                header.insert_header("Location", new_url).map_err(|e| {
                    Error::explain(
                        ErrorType::InternalError,
                        format!("Failed to insert Location header: {}", e),
                    )
                })?;
                header.insert_header("Content-Length", "0").map_err(|e| {
                    Error::explain(
                        ErrorType::InternalError,
                        format!("Failed to insert Content-Length header: {}", e),
                    )
                })?;

                session
                    .write_response_header(Box::new(header), true)
                    .await?;
                return Ok(FilterResult::Handled);
            }
        }
        Ok(FilterResult::Continue)
    }
}
