use crate::state::{AppState, HostConfig, LocationConfig};
use async_trait::async_trait;
use pingora::prelude::*;
use pingora::http::ResponseHeader;
use http::header::HeaderName; 
use std::sync::Arc;
use std::sync::atomic::Ordering;
use bytes::Bytes;
use std::time::Duration;
use crate::auth; 
use base64::{Engine as _, engine::general_purpose};

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
            if let Some(key_auth) = self.state.get_acme_challenge(token) {
                let mut header = ResponseHeader::build(200, Some(4)).unwrap();
                header.insert_header("Content-Type", "text/plain").unwrap();
                let body_bytes = Bytes::from(key_auth);
                header.insert_header("Content-Length", body_bytes.len().to_string()).unwrap();
                
                session.write_response_header(Box::new(header), false).await?;
                session.write_response_body(Some(body_bytes), true).await?;
                return Ok(true); 
            } else {
                let _ = session.respond_error(404).await;
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

        // 3. 설정 조회 및 Access List 검사
        if let Some(host_config) = self.state.get_host_config(&host) {
            ctx.host_config = Some(host_config.clone());

            // 🔥 [Access List 검사 로직 시작] 🔥
            if let Some(acl_id) = host_config.access_list_id {
                // ACL ID가 설정되어 있다면 검사 수행
                if let Some(acl) = self.state.get_access_list(acl_id) {
                    
                    // (A) IP 기반 필터링
                    if !acl.ips.is_empty() {
                        // 클라이언트 IP 추출 (포트 제거)
                        let client_ip = session.client_addr()
                            .map(|a| a.to_string())
                            .unwrap_or_default();
                        let client_ip = client_ip.split(':').next().unwrap_or(&client_ip).to_string();

                        let mut ip_allowed = true; // 기본값
                        let mut has_allow_rules = false;

                        for rule in &acl.ips {
                            if rule.action == "allow" {
                                has_allow_rules = true;
                                if rule.ip == client_ip {
                                    ip_allowed = true;
                                    break; // 허용 규칙에 맞으면 즉시 통과
                                } else {
                                    ip_allowed = false; // Allow 규칙이 하나라도 있으면 기본은 Deny
                                }
                            } else if rule.action == "deny" {
                                if rule.ip == client_ip {
                                    tracing::warn!("⛔ Access Denied (IP Blocked): {} -> {}", client_ip, ctx.host);
                                    let _ = session.respond_error(403).await;
                                    return Ok(true); // 요청 거부
                                }
                            }
                        }

                        // Allow 규칙이 존재하는데 매칭되지 않은 경우
                        if has_allow_rules && !ip_allowed {
                            tracing::warn!("⛔ Access Denied (IP Not Allowed): {} -> {}", client_ip, ctx.host);
                            let _ = session.respond_error(403).await;
                            return Ok(true); // 요청 거부
                        }
                    }

                    // (B) Basic Auth (사용자 인증)
                    if !acl.clients.is_empty() {
                        let auth_header = session.req_header().headers.get("Authorization");
                        let mut authenticated = false;

                        if let Some(value) = auth_header {
                            if let Ok(v_str) = value.to_str() {
                                if v_str.starts_with("Basic ") {
                                    let encoded = &v_str[6..];
                                    // Base64 디코딩
                                    if let Ok(decoded) = general_purpose::STANDARD.decode(encoded) {
                                        if let Ok(creds) = String::from_utf8(decoded) {
                                            if let Some((username, password)) = creds.split_once(':') {
                                                // 사용자 찾기 및 비밀번호 검증
                                                if let Some(client_conf) = acl.clients.iter().find(|c| c.username == username) {
                                                    if auth::verify_password(password, &client_conf.password_hash) {
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
                            // 인증 실패 시 로그인 창 팝업 (401)
                            tracing::info!("🔒 Authentication required for {}", ctx.host);
                            let mut header = ResponseHeader::build(401, Some(4)).unwrap();
                            header.insert_header("WWW-Authenticate", "Basic realm=\"Restricted Area\"").unwrap();
                            header.insert_header("Content-Length", "0").unwrap();
                            session.write_response_header(Box::new(header), true).await?;
                            return Ok(true); // 요청 처리 중단 (클라이언트는 로그인 창 띄움)
                        }
                    }
                }
            }
            // 🔥 [Access List 검사 로직 끝] 🔥


            // 4. Redirection Host Check
            if let Some(redirect_target) = &host_config.redirect_to {
                 let status = host_config.redirect_status;
                 let query = session.req_header().uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
                 
                 let target = if redirect_target.ends_with('/') && path.starts_with('/') {
                     &redirect_target[..redirect_target.len()-1]
                 } else {
                     redirect_target
                 };
                 
                 let new_url = format!("{}{}{}", target, path, query);
                 let mut header = ResponseHeader::build(status, Some(4)).unwrap();
                 header.insert_header("Location", new_url).unwrap();
                 header.insert_header("Content-Length", "0").unwrap();
                 
                 session.write_response_header(Box::new(header), true).await?;
                 return Ok(true);
            }

            // 5. SSL Forced Redirect
            let is_tls = session
                .server_addr()
                .map(|a| a.as_inet().map(|s| s.port() == 443).unwrap_or(false))
                .unwrap_or(false);

            if host_config.ssl_forced && !is_tls {
                let query = session.req_header().uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
                let new_url = format!("https://{}{}{}", host, path, query);

                let mut header = ResponseHeader::build(301, Some(4)).unwrap();
                header.insert_header("Location", new_url).unwrap();
                header.insert_header("Content-Length", "0").unwrap();
                
                session.write_response_header(Box::new(header), true).await?;
                return Ok(true);
            }

            // 6. Location Matching
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
        // ... (Path Rewrite 로직은 기존 유지) ...
        if let Some(loc) = &ctx.matched_location {
            // (Path Rewrite 내용 생략 - 기존 코드 그대로 두세요)
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

                    let _ = upstream_request.set_uri(new_uri.parse().unwrap());
                    tracing::info!("🔄 Rewrote path: {} -> {}", original_path, new_path);
                }
            }
        }

        // 2. Custom Request Headers (수정됨)
        if let Some(host_config) = &ctx.host_config {
            let headers = self.state.get_headers(host_config.id);
            for h in headers {
                if h.target == "request" {
                    // 👇 String -> HeaderName 안전하게 변환
                    if let Ok(header_name) = HeaderName::from_bytes(h.name.as_bytes()) {
                        // remove_header도 HeaderName을 받으면 안전합니다.
                        let _ = upstream_request.remove_header(&header_name);
                        // insert_header에 소유권이 있는 header_name을 넘김
                        upstream_request.insert_header(header_name, &h.value).unwrap();
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
                    // 👇 String -> HeaderName 안전하게 변환
                    if let Ok(header_name) = HeaderName::from_bytes(h.name.as_bytes()) {
                        let _ = upstream_response.remove_header(&header_name);
                        upstream_response.insert_header(header_name, &h.value).unwrap();
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