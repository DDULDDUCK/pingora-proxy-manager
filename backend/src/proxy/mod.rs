pub mod connection_filter;
pub mod filters;

use self::filters::{FilterResult, ProxyFilter};
use crate::constants;
use crate::state::{AppState, HostConfig, LocationConfig};
use async_trait::async_trait;
use http::header::HeaderName;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use rand::prelude::IndexedRandom;
use std::sync::atomic::Ordering;
use std::sync::{Arc, OnceLock};
use std::time::Duration; // Fix for rand 0.9

pub struct DynamicProxy {
    pub state: Arc<AppState>,
}

fn upstream_body_throttle() -> Option<Duration> {
    static THROTTLE: OnceLock<Option<Duration>> = OnceLock::new();
    *THROTTLE.get_or_init(|| {
        std::env::var("PPM_UPSTREAM_BODY_THROTTLE_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_millis)
    })
}

pub struct ProxyCtx {
    pub host: String,
    pub host_config: Option<HostConfig>,
    pub matched_location: Option<LocationConfig>,
    pub retry_count: usize,
    pub attempted_targets: Vec<String>,
}

fn configure_upstream_timeouts(peer: &mut HttpPeer, is_upgrade_request: bool) {
    peer.options.connection_timeout =
        Some(Duration::from_millis(constants::timeout::CONNECTION_MS));

    if is_upgrade_request {
        peer.options.read_timeout = None;
        peer.options.write_timeout = None;
    } else {
        peer.options.read_timeout = Some(Duration::from_secs(constants::timeout::READ_SECS));
        peer.options.write_timeout = Some(Duration::from_secs(constants::timeout::WRITE_SECS));
    }
}

#[async_trait]
impl ProxyHttp for DynamicProxy {
    type CTX = ProxyCtx;

    fn new_ctx(&self) -> Self::CTX {
        ProxyCtx {
            host: String::new(),
            host_config: None,
            matched_location: None,
            retry_count: 0,
            attempted_targets: Vec::new(),
        }
    }

    fn fail_to_connect(
        &self,
        _session: &mut Session,
        _peer: &HttpPeer,
        ctx: &mut Self::CTX,
        mut e: Box<Error>,
    ) -> Box<Error> {
        let target_count = if let Some(loc) = &ctx.matched_location {
            loc.targets.len()
        } else if let Some(host) = &ctx.host_config {
            host.targets.len()
        } else {
            0
        };

        const MAX_RETRIES: usize = 2;
        if target_count > 1 && ctx.retry_count < MAX_RETRIES {
            ctx.retry_count += 1;
            e.set_retry(true);
            tracing::warn!(
                "Upstream connect failed for host {}. Retrying with another target (attempt {}/{})",
                ctx.host,
                ctx.retry_count,
                MAX_RETRIES
            );
        }

        e
    }

    /// 요청 필터링: ACME Challenge 처리 및 라우팅 정보 조회
    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // 1. 초기 필터 (Host 정보 없이 가능한 것들)
        let early_filters: Vec<Box<dyn ProxyFilter>> = vec![Box::new(filters::acme::AcmeFilter)];

        for filter in early_filters {
            if let FilterResult::Handled = filter.request_filter(session, ctx).await? {
                return Ok(true);
            }
        }

        // 2. Host 파싱
        // Prioritize URI host (handles HTTP/2 :authority and HTTP/1.1 parsed host)
        let host = if let Some(h) = session.req_header().uri.host() {
            h.to_string()
        } else {
            // Fallback to manual Host header parsing
            session
                .req_header()
                .headers
                .get("Host")
                .and_then(|h| h.to_str().ok())
                .unwrap_or_default()
                .split(':')
                .next()
                .unwrap_or_default()
                .to_string()
        };

        ctx.host = host.clone();

        // 3. 설정 조회 및 호스트 기반 필터
        if let Some(host_config) = self.state.get_host_config(&host) {
            ctx.host_config = Some(host_config.clone());

            let host_filters: Vec<Box<dyn ProxyFilter>> = vec![
                Box::new(filters::ssl::SslFilter),
                Box::new(filters::redirect::RedirectFilter),
                Box::new(filters::acl::AclFilter {
                    state: self.state.clone(),
                }),
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
                    let rewritten_path = if original_path.len() == loc.path.len() {
                        "/".to_string()
                    } else {
                        let suffix = &original_path[loc.path.len()..];
                        if suffix.starts_with('/') {
                            suffix.to_string()
                        } else {
                            format!("/{}", suffix)
                        }
                    };

                    let new_uri = if let Some(query) = session.req_header().uri.query() {
                        format!("{}?{}", rewritten_path, query)
                    } else {
                        rewritten_path.clone()
                    };

                    upstream_request.set_uri(new_uri.parse().map_err(|e| {
                        Error::explain(
                            ErrorType::InternalError,
                            format!("Failed to parse URI: {}", e),
                        )
                    })?);
                    tracing::info!("🔄 Rewrote path: {} -> {}", original_path, rewritten_path);
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
                            .map_err(|e| {
                                Error::explain(
                                    ErrorType::InternalError,
                                    format!("Failed to insert request header: {}", e),
                                )
                            })?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 업스트림 응답 필터링 (Response Headers 수행)
    async fn upstream_response_filter(
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
                            .map_err(|e| {
                                Error::explain(
                                    ErrorType::InternalError,
                                    format!("Failed to insert response header: {}", e),
                                )
                            })?;
                    }
                }
            }
        }
        Ok(())
    }

    fn upstream_response_body_filter(
        &self,
        _session: &mut Session,
        _body: &mut Option<bytes::Bytes>,
        _end_of_stream: bool,
        _ctx: &mut Self::CTX,
    ) -> Result<Option<Duration>> {
        Ok(upstream_body_throttle())
    }

    /// 실제 라우팅 로직 (CTX 활용)
    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        if let Some(host_config) = &ctx.host_config {
            // LOAD BALANCING LOGIC:
            // Check matched_location first, then host_config.
            // Both now support multiple targets (Vec<String>).

            let (targets, scheme, verify_ssl, upstream_sni) =
                if let Some(loc) = &ctx.matched_location {
                    (
                        &loc.targets,
                        &loc.scheme,
                        loc.verify_ssl,
                        loc.upstream_sni.as_ref(),
                    )
                } else {
                    (
                        &host_config.targets,
                        &host_config.scheme,
                        host_config.verify_ssl,
                        host_config.upstream_sni.as_ref(),
                    )
                };

            // Select a target using simple Random Load Balancing
            // If targets is empty (shouldn't happen with valid config), error out.
            let target = if targets.is_empty() {
                tracing::error!("No upstream targets configured for host: {}", ctx.host);
                return Err(Error::explain(
                    ErrorType::HTTPStatus(constants::http::INTERNAL_ERROR),
                    "No upstream targets found",
                ));
            } else {
                let candidates: Vec<&String> = targets
                    .iter()
                    .filter(|t| !ctx.attempted_targets.iter().any(|picked| picked == *t))
                    .collect();

                let mut rng = rand::rng(); // Fixed for rand 0.9
                if let Some(next) = candidates.choose(&mut rng) {
                    (*next).clone()
                } else {
                    targets.choose(&mut rng).cloned().ok_or_else(|| {
                        Error::explain(
                            ErrorType::HTTPStatus(constants::http::INTERNAL_ERROR),
                            "No selectable upstream targets found",
                        )
                    })?
                }
            };

            ctx.attempted_targets.push(target.clone());

            let use_tls = scheme == "https";
            let sni = upstream_sni.cloned().unwrap_or_else(|| ctx.host.clone());
            let is_upgrade_request = session.is_upgrade_req();

            tracing::info!(
                "Routing {} -> {} (LB: Random/{} targets, TLS: {}, VerifySSL: {}, SNI: {}, Upgrade: {})",
                ctx.host,
                target,
                targets.len(),
                use_tls,
                verify_ssl,
                sni,
                is_upgrade_request,
            );

            let mut peer = Box::new(HttpPeer::new(target, use_tls, sni.clone()));

            if use_tls {
                peer.sni = sni;
                peer.options.verify_cert = verify_ssl;
                peer.options.verify_hostname = verify_ssl;
                peer.options.upstream_tls_handshake_complete_hook = Some(Arc::new(|_| {
                    Some(Arc::new("upstream_tls".to_string())
                        as Arc<dyn std::any::Any + Send + Sync>)
                }));
            }

            configure_upstream_timeouts(&mut peer, is_upgrade_request);

            return Ok(peer);
        }

        tracing::warn!("No route found for host: {}", ctx.host);
        Err(Error::explain(
            ErrorType::HTTPStatus(constants::http::NOT_FOUND),
            "Host not found",
        ))
    }

    /// 요청 로깅 및 통계 집계 (응답 전송 후 호출됨)
    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        self.state
            .metrics
            .total_requests
            .fetch_add(1, Ordering::Relaxed);

        if let Some(resp) = session.response_written() {
            let status = resp.status.as_u16();
            let body_len = session.body_bytes_sent() as u64;
            let upstream_body_len = session.upstream_body_bytes_received() as u64;

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
                upstream_bytes = upstream_body_len,
                host = %ctx.host,
                "Request handled"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ProxyConfig;
    use std::collections::HashMap;
    use std::thread;
    use std::time::Instant;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::runtime::Builder;
    use tokio::time::{sleep, timeout};

    const TEST_PROXY_ADDR: &str = "127.0.0.1:38080";
    const TEST_UPGRADE_ORIGIN_ADDR: &str = "127.0.0.1:39284";
    const TEST_HANGING_ORIGIN_ADDR: &str = "127.0.0.1:39285";

    fn init_test_stack() {
        static INIT: OnceLock<()> = OnceLock::new();

        INIT.get_or_init(|| {
            spawn_upgrade_origin();
            spawn_hanging_origin();
            spawn_proxy();
            thread::sleep(Duration::from_millis(400));
        });
    }

    fn spawn_proxy() {
        let state = Arc::new(AppState::new());
        state.update_config(test_proxy_config());

        thread::spawn(move || {
            let mut server = Server::new(None).expect("create Pingora test server");
            server.bootstrap();

            let mut proxy = http_proxy_service(&server.configuration, DynamicProxy { state });
            proxy.add_tcp(TEST_PROXY_ADDR);

            server.add_service(proxy);
            server.run_forever();
        });
    }

    fn spawn_upgrade_origin() {
        thread::spawn(|| {
            let runtime = Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build upgrade origin runtime");

            runtime.block_on(async {
                let listener = TcpListener::bind(TEST_UPGRADE_ORIGIN_ADDR)
                    .await
                    .expect("bind upgrade origin");

                loop {
                    let (stream, _) = listener.accept().await.expect("accept upgrade origin conn");
                    tokio::spawn(async move {
                        let _ = handle_upgrade_origin(stream).await;
                    });
                }
            });
        });
    }

    fn spawn_hanging_origin() {
        thread::spawn(|| {
            let runtime = Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build hanging origin runtime");

            runtime.block_on(async {
                let listener = TcpListener::bind(TEST_HANGING_ORIGIN_ADDR)
                    .await
                    .expect("bind hanging origin");

                loop {
                    let (stream, _) = listener.accept().await.expect("accept hanging origin conn");
                    tokio::spawn(async move {
                        let _ = handle_hanging_origin(stream).await;
                    });
                }
            });
        });
    }

    fn test_proxy_config() -> ProxyConfig {
        let mut hosts = HashMap::new();

        hosts.insert(
            "upgrade.local".to_string(),
            HostConfig {
                id: 1,
                targets: vec![TEST_UPGRADE_ORIGIN_ADDR.to_string()],
                scheme: "http".to_string(),
                locations: vec![],
                ssl_forced: false,
                verify_ssl: true,
                upstream_sni: None,
                redirect_to: None,
                redirect_status: 301,
                access_list_id: None,
                headers: vec![],
            },
        );

        hosts.insert(
            "location.local".to_string(),
            HostConfig {
                id: 2,
                targets: vec![TEST_HANGING_ORIGIN_ADDR.to_string()],
                scheme: "http".to_string(),
                locations: vec![LocationConfig {
                    path: "/api/socket.io/".to_string(),
                    targets: vec![TEST_UPGRADE_ORIGIN_ADDR.to_string()],
                    scheme: "http".to_string(),
                    rewrite: false,
                    verify_ssl: true,
                    upstream_sni: None,
                }],
                ssl_forced: false,
                verify_ssl: true,
                upstream_sni: None,
                redirect_to: None,
                redirect_status: 301,
                access_list_id: None,
                headers: vec![],
            },
        );

        hosts.insert(
            "hang.local".to_string(),
            HostConfig {
                id: 3,
                targets: vec![TEST_HANGING_ORIGIN_ADDR.to_string()],
                scheme: "http".to_string(),
                locations: vec![],
                ssl_forced: false,
                verify_ssl: true,
                upstream_sni: None,
                redirect_to: None,
                redirect_status: 301,
                access_list_id: None,
                headers: vec![],
            },
        );

        ProxyConfig {
            hosts,
            access_lists: HashMap::new(),
            headers: HashMap::new(),
        }
    }

    async fn handle_upgrade_origin(mut stream: TcpStream) -> std::io::Result<()> {
        let _ = read_until_header_end(&mut stream).await?;
        stream
            .write_all(
                b"HTTP/1.1 101 Switching Protocols\r\nConnection: upgrade\r\nUpgrade: websocket\r\n\r\n",
            )
            .await?;

        let mut buf = [0_u8; 1024];
        loop {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                return Ok(());
            }
            stream.write_all(&buf[..n]).await?;
        }
    }

    async fn handle_hanging_origin(mut stream: TcpStream) -> std::io::Result<()> {
        let _ = read_until_header_end(&mut stream).await?;
        sleep(Duration::from_secs(constants::timeout::READ_SECS + 5)).await;
        Ok(())
    }

    async fn read_until_header_end(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut buf = [0_u8; 1024];

        loop {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                return Ok(data);
            }

            data.extend_from_slice(&buf[..n]);
            if data.windows(4).any(|window| window == b"\r\n\r\n") {
                return Ok(data);
            }
        }
    }

    async fn read_response_header(stream: &mut TcpStream) -> std::io::Result<(u16, Vec<u8>)> {
        let response = read_until_header_end(stream).await?;
        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("response header terminator");

        let header = std::str::from_utf8(&response[..header_end]).expect("utf8 response header");
        let status = header
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|code| code.parse::<u16>().ok())
            .expect("status code");

        Ok((status, response[header_end..].to_vec()))
    }

    async fn send_upgrade_request(host: &str, path: &str) -> TcpStream {
        let mut stream = TcpStream::connect(TEST_PROXY_ADDR)
            .await
            .expect("connect to test proxy");
        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n"
        );

        stream
            .write_all(request.as_bytes())
            .await
            .expect("write upgrade request");
        stream.flush().await.expect("flush upgrade request");

        let (status, preread) = timeout(Duration::from_secs(5), read_response_header(&mut stream))
            .await
            .expect("timely upgrade response")
            .expect("read upgrade response header");

        assert_eq!(status, 101);
        assert!(
            preread.is_empty(),
            "upgrade response should not include preread body"
        );

        stream
    }

    #[test]
    fn upgraded_requests_disable_short_upstream_timeouts() {
        let mut peer = HttpPeer::new("127.0.0.1:80", false, String::new());

        configure_upstream_timeouts(&mut peer, true);

        assert_eq!(
            peer.options.connection_timeout,
            Some(Duration::from_millis(constants::timeout::CONNECTION_MS))
        );
        assert!(peer.options.read_timeout.is_none());
        assert!(peer.options.write_timeout.is_none());
    }

    #[test]
    fn standard_requests_keep_default_upstream_timeouts() {
        let mut peer = HttpPeer::new("127.0.0.1:80", false, String::new());

        configure_upstream_timeouts(&mut peer, false);

        assert_eq!(
            peer.options.connection_timeout,
            Some(Duration::from_millis(constants::timeout::CONNECTION_MS))
        );
        assert_eq!(
            peer.options.read_timeout,
            Some(Duration::from_secs(constants::timeout::READ_SECS))
        );
        assert_eq!(
            peer.options.write_timeout,
            Some(Duration::from_secs(constants::timeout::WRITE_SECS))
        );
    }

    #[tokio::test]
    async fn upgraded_host_route_survives_idle_beyond_default_timeout() {
        init_test_stack();

        let mut stream =
            send_upgrade_request("upgrade.local", "/socket.io/?EIO=4&transport=websocket").await;

        sleep(Duration::from_secs(constants::timeout::READ_SECS + 1)).await;

        stream
            .write_all(b"ping")
            .await
            .expect("write upgraded body");
        stream.flush().await.expect("flush upgraded body");

        let mut echo = [0_u8; 4];
        timeout(Duration::from_secs(5), stream.read_exact(&mut echo))
            .await
            .expect("echo should arrive")
            .expect("read echoed upgraded bytes");

        assert_eq!(&echo, b"ping");
    }

    #[tokio::test]
    async fn upgraded_location_route_survives_idle_beyond_default_timeout() {
        init_test_stack();

        let mut stream = send_upgrade_request(
            "location.local",
            "/api/socket.io/?EIO=4&transport=websocket",
        )
        .await;

        sleep(Duration::from_secs(constants::timeout::READ_SECS + 1)).await;

        stream
            .write_all(b"pong")
            .await
            .expect("write upgraded location body");
        stream.flush().await.expect("flush upgraded location body");

        let mut echo = [0_u8; 4];
        timeout(Duration::from_secs(5), stream.read_exact(&mut echo))
            .await
            .expect("location echo should arrive")
            .expect("read echoed location upgraded bytes");

        assert_eq!(&echo, b"pong");
    }

    #[tokio::test]
    async fn normal_http_requests_still_timeout_when_origin_never_responds() {
        init_test_stack();

        let mut stream = TcpStream::connect(TEST_PROXY_ADDR)
            .await
            .expect("connect to test proxy");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: hang.local\r\nConnection: close\r\n\r\n")
            .await
            .expect("write plain http request");
        stream.flush().await.expect("flush plain http request");

        let started_at = Instant::now();
        let (status, _) = timeout(
            Duration::from_secs(constants::timeout::READ_SECS + 5),
            read_response_header(&mut stream),
        )
        .await
        .expect("timeout response should arrive")
        .expect("read timeout response header");

        assert!(
            started_at.elapsed()
                >= Duration::from_secs(constants::timeout::READ_SECS.saturating_sub(1)),
            "plain HTTP timeout should still wait roughly the default read timeout"
        );
        assert_eq!(status, 502);
    }
}
