use crate::state::{AppState, HostConfig, LocationConfig};
use async_trait::async_trait;
use pingora::prelude::*;
use pingora::http::ResponseHeader;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use bytes::Bytes;
use std::time::Duration;

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
        let path = session.req_header().uri.path();
        
        // 1. ACME Challenge 처리
        if path.starts_with("/.well-known/acme-challenge/") {
            let token = path.trim_start_matches("/.well-known/acme-challenge/");
            tracing::info!("📢 ACME Challenge received for token: {}", token);

            if let Some(key_auth) = self.state.get_acme_challenge(token) {
                let mut header = ResponseHeader::build(200, Some(4)).unwrap();
                header.insert_header("Content-Type", "text/plain").unwrap();
                let body_bytes = Bytes::from(key_auth);
                header.insert_header("Content-Length", body_bytes.len().to_string()).unwrap();
                
                session.write_response_header(Box::new(header), false).await?;
                session.write_response_body(Some(body_bytes), true).await?;
                return Ok(true); // 요청 처리 완료
            } else {
                tracing::warn!("⚠️ Unknown ACME token: {}", token);
                let _ = session.respond_error(404).await;
                return Ok(true);
            }
        }

        // 2. Host 파싱 및 설정 조회 (Context에 저장)
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

        if let Some(host_config) = self.state.get_host_config(&host) {
            ctx.host_config = Some(host_config.clone());

            // Location Matching (Longest Prefix Match)
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
        
        Ok(false) // 일반 요청은 계속 진행
    }

    /// 업스트림 요청 전 필터링 (Path Rewrite 수행)
    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        _upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        // ctx.matched_location이 있고 rewrite 옵션이 켜져 있다면 경로 재작성
        if let Some(loc) = &ctx.matched_location {
            if loc.rewrite {
                let original_path = session.req_header().uri.path().to_string();
                if original_path.starts_with(&loc.path) {
                    // Prefix 제거
                    let new_path = if original_path.len() == loc.path.len() {
                        "/"
                    } else {
                        &original_path[loc.path.len()..]
                    };
                    
                    // 쿼리 스트링 보존
                    let new_uri = if let Some(query) = session.req_header().uri.query() {
                        format!("{}?{}", new_path, query)
                    } else {
                        new_path.to_string()
                    };

                    // URI 업데이트 (RequestHeader 수정)
                    let _ = session.req_header_mut().set_uri(new_uri.parse().unwrap());
                    tracing::info!("🔄 Rewrote path: {} -> {}", original_path, new_path);
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
            
            let mut peer = Box::new(HttpPeer::new(
                target, 
                use_tls,
                ctx.host.clone()
            ));
            
            if use_tls {
                peer.sni = ctx.host.clone();
            }

            peer.options.connection_timeout = Some(Duration::from_millis(500));
            peer.options.read_timeout = Some(Duration::from_secs(10));
            peer.options.write_timeout = Some(Duration::from_secs(5));

            return Ok(peer);
        }

        tracing::warn!("No route found for host: {}", ctx.host);
        Err(Error::explain(ErrorType::HTTPStatus(404), "Host not found"))
    }

    /// 요청 로깅 및 통계 집계 (응답 전송 후 호출됨)
    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        _ctx: &mut Self::CTX,
    ) {
        // 1. 통계 업데이트
        self.state.metrics.total_requests.fetch_add(1, Ordering::Relaxed);
        
        if let Some(resp) = session.response_written() {
            let status = resp.status.as_u16();
            let body_len = session.body_bytes_sent() as u64;

            self.state.metrics.total_bytes.fetch_add(body_len, Ordering::Relaxed);

            if status >= 200 && status < 300 {
                self.state.metrics.status_2xx.fetch_add(1, Ordering::Relaxed);
            } else if status >= 400 && status < 500 {
                self.state.metrics.status_4xx.fetch_add(1, Ordering::Relaxed);
            } else if status >= 500 {
                self.state.metrics.status_5xx.fetch_add(1, Ordering::Relaxed);
            }

            // 2. 액세스 로그
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
