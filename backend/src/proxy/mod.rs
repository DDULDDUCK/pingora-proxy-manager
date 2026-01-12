pub mod filters;

use crate::constants;
use crate::state::{AppState, HostConfig, LocationConfig};
use async_trait::async_trait;
use http::header::HeaderName;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use self::filters::{ProxyFilter, FilterResult};

pub struct DynamicProxy {
    pub state: Arc<AppState>,
}

pub struct ProxyCtx {
    pub host: String,
    pub host_config: Option<HostConfig>,
    pub matched_location: Option<LocationConfig>,
}

#[async_trait]
impl ProxyHttp for DynamicProxy {
    type CTX = ProxyCtx;

    fn new_ctx(&self) -> Self::CTX {
        ProxyCtx {
            host: String::new(),
            host_config: None,
            matched_location: None,
        }
    }

    /// 요청 필터링: ACME Challenge 처리 및 라우팅 정보 조회
    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // 1. 초기 필터 (Host 정보 없이 가능한 것들)
        let early_filters: Vec<Box<dyn ProxyFilter>> = vec![
            Box::new(filters::acme::AcmeFilter),
        ];

        for filter in early_filters {
            if let FilterResult::Handled = filter.request_filter(session, ctx).await? {
                return Ok(true);
            }
        }

        // 2. Host 파싱
        let host = session
            .req_header()
            .headers
            .get("Host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            .split(':')
            .next()
            .unwrap_or_default()
            .to_string();

        ctx.host = host.clone();

        // 3. 설정 조회 및 호스트 기반 필터
        if let Some(host_config) = self.state.get_host_config(&host) {
            ctx.host_config = Some(host_config.clone());

            let host_filters: Vec<Box<dyn ProxyFilter>> = vec![
                Box::new(filters::acl::AclFilter { state: self.state.clone() }),
                Box::new(filters::redirect::RedirectFilter),
                Box::new(filters::ssl::SslFilter),
            ];

            for filter in host_filters {
                if let FilterResult::Handled = filter.request_filter(session, ctx).await? {
                    return Ok(true);
                }
            }

            // 4. Location Matching
            let path = session.req_header().uri.path();
            let mut best_match_len = 0;
            let mut matched_loc = None;

            for loc in &host_config.locations {
                if path.starts_with(&loc.path) && loc.path.len() > best_match_len {
                    matched_loc = Some(loc.clone());
                    best_match_len = loc.path.len();
                }
            }
            ctx.matched_location = matched_loc;
        }

        Ok(false)
    }

    /// 업스트림 요청 전 필터링 (Path Rewrite 및 Request Headers 수행)
    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if let Some(loc) = &ctx.matched_location {
            if loc.rewrite {
                let original_path = session.req_header().uri.path().to_string();
                if original_path.starts_with(&loc.path) {
                    let new_path = if original_path.len() == loc.path.len() {
                        "/"
                    } else {
                        &original_path[loc.path.len()..]
                    };

                    let new_uri = if let Some(query) = session.req_header().uri.query() {
                        format!("{}?{}", new_path, query)
                    } else {
                        new_path.to_string()
                    };

                    upstream_request.set_uri(new_uri.parse().map_err(|e| Error::explain(ErrorType::InternalError, format!("Failed to parse URI: {}", e)))?);
                    tracing::info!("🔄 Rewrote path: {} -> {}", original_path, new_path);
                }
            }
        }

        // 2. Custom Request Headers
        if let Some(host_config) = &ctx.host_config {
            let headers = self.state.get_headers(host_config.id);
            for h in headers {
                if h.target == "request" {
                    if let Ok(header_name) = HeaderName::from_bytes(h.name.as_bytes()) {
                        let _ = upstream_request.remove_header(&header_name);
                        upstream_request
                            .insert_header(header_name, &h.value)
                            .map_err(|e| Error::explain(ErrorType::InternalError, format!("Failed to insert request header: {}", e)))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 업스트림 응답 필터링 (Response Headers 수행)
    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if let Some(host_config) = &ctx.host_config {
            let headers = self.state.get_headers(host_config.id);
            for h in headers {
                if h.target == "response" {
                    if let Ok(header_name) = HeaderName::from_bytes(h.name.as_bytes()) {
                        let _ = upstream_response.remove_header(&header_name);
                        upstream_response
                            .insert_header(header_name, &h.value)
                            .map_err(|e| Error::explain(ErrorType::InternalError, format!("Failed to insert response header: {}", e)))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 실제 라우팅 로직 (CTX 활용)
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        if let Some(host_config) = &ctx.host_config {
            let (target, scheme) = if let Some(loc) = &ctx.matched_location {
                (&loc.target, &loc.scheme)
            } else {
                (&host_config.target, &host_config.scheme)
            };

            let use_tls = scheme == "https";
            tracing::info!("Routing {} -> {} (TLS: {})", ctx.host, target, use_tls);

            let mut peer = Box::new(HttpPeer::new(target, use_tls, ctx.host.clone()));

            if use_tls {
                peer.sni = ctx.host.clone();
            }

            peer.options.connection_timeout = Some(Duration::from_millis(constants::timeout::CONNECTION_MS));
            peer.options.read_timeout = Some(Duration::from_secs(constants::timeout::READ_SECS));
            peer.options.write_timeout = Some(Duration::from_secs(constants::timeout::WRITE_SECS));

            return Ok(peer);
        }

        tracing::warn!("No route found for host: {}", ctx.host);
        Err(Error::explain(ErrorType::HTTPStatus(constants::http::NOT_FOUND), "Host not found"))
    }

    /// 요청 로깅 및 통계 집계 (응답 전송 후 호출됨)
    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        _ctx: &mut Self::CTX,
    ) {
        self.state
            .metrics
            .total_requests
            .fetch_add(1, Ordering::Relaxed);

        if let Some(resp) = session.response_written() {
            let status = resp.status.as_u16();
            let body_len = session.body_bytes_sent() as u64;

            self.state
                .metrics
                .total_bytes
                .fetch_add(body_len, Ordering::Relaxed);

            if status >= constants::http::OK && status < 300 {
                self.state
                    .metrics
                    .status_2xx
                    .fetch_add(1, Ordering::Relaxed);
            } else if status >= 400 && status < 500 {
                self.state
                    .metrics
                    .status_4xx
                    .fetch_add(1, Ordering::Relaxed);
            } else if status >= 500 {
                self.state
                    .metrics
                    .status_5xx
                    .fetch_add(1, Ordering::Relaxed);
            }

            tracing::info!(
                target: "access_log",
                method = %session.req_header().method,
                path = %session.req_header().uri.path(),
                status = status,
                bytes = body_len,
                host = ?session.req_header().headers.get("Host"),
                "Request handled"
            );
        }
    }
}
