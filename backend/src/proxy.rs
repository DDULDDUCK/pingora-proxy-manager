use crate::state::AppState;
use async_trait::async_trait;
use pingora::prelude::*;
use pingora::http::ResponseHeader;
use std::sync::Arc;
use bytes::Bytes;

pub struct DynamicProxy {
    pub state: Arc<AppState>,
}

#[async_trait]
impl ProxyHttp for DynamicProxy {
    /// 요청마다의 컨텍스트 (필요하다면 여기에 로깅 정보 등을 담음)
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    /// 요청 필터링: ACME Challenge 처리
    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        let path = session.req_header().uri.path();
        
        if path.starts_with("/.well-known/acme-challenge/") {
            let token = path.trim_start_matches("/.well-known/acme-challenge/");
            tracing::info!("📢 ACME Challenge received for token: {}", token);

            if let Some(key_auth) = self.state.get_acme_challenge(token) {
                let mut header = ResponseHeader::build(200, Some(4)).unwrap();
                header.insert_header("Content-Type", "text/plain").unwrap();
                let body_bytes = Bytes::from(key_auth);
                header.insert_header("Content-Length", body_bytes.len().to_string()).unwrap();
                
                // 헤더 전송 (스트림 안 끝남)
                session.write_response_header(Box::new(header), false).await?;
                // 바디 전송 (스트림 끝남)
                session.write_response_body(Some(body_bytes), true).await?;
                return Ok(true); // 요청 처리 완료
            } else {
                tracing::warn!("⚠️ Unknown ACME token: {}", token);
                let _ = session.respond_error(404).await;
                return Ok(true);
            }
        }
        
        Ok(false) // 일반 요청은 계속 진행
    }

    /// 실제 라우팅 로직
    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // 1. Host 헤더 파싱
        let host = session
            .req_header()
            .headers
            .get("Host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            // 포트 번호 제거 (예: example.com:8080 -> example.com)
            .split(':')
            .next()
            .unwrap_or_default();

        // 2. 상태(State)에서 라우팅 조회 (Lock-Free Fast Path)
        if let Some(host_config) = self.state.get_host_config(host) {
            let use_tls = host_config.scheme == "https";
            tracing::info!("Routing {} -> {} (TLS: {})", host, host_config.target, use_tls);
            
            let mut peer = Box::new(HttpPeer::new(
                host_config.target, 
                use_tls,
                host.to_string()
            ));
            
            // HTTPS 업스트림인 경우 SNI 설정 (필요시)
            if use_tls {
                // Pingora 0.6 PeerOptions: verify_cert는 필드일 수 있음.
                // 만약 private이라면 생성자에서 처리해야 함.
                // HttpPeer::new() 시점에 옵션을 다 넣을 순 없음.
                
                // 시도 1: 필드 직접 접근 (verify_cert)
                // peer.options.verify_cert = false; 
                
                // 시도 2: sni 설정 (보통 이걸 해야 함)
                peer.sni = host.to_string();
                
                // Pingora에서 TLS 검증을 끄는 건 보안상 위험하지만, 사용자가 원할 수 있음.
                // 여기서는 verify_cert 메서드가 없다고 하므로 일단 주석 처리하고
                // SNI만 설정합니다. (SNI가 없으면 핸드셰이크 실패할 수 있음)
                // peer.options.verify_cert = false; 
            }

            return Ok(peer);
        }

        // 3. 매칭되는 호스트가 없을 경우
        tracing::warn!("No route found for host: {}", host);
        Err(Error::explain(ErrorType::HTTPStatus(404), "Host not found"))
    }
}
