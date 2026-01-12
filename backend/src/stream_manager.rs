use crate::constants;
use crate::db::{self, DbPool};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::task::JoinHandle;
use tokio::time::timeout;

#[derive(Clone)]
pub struct StreamManager {
    pub db_pool: DbPool,
    // Map listen_port -> Task Handle
    pub tasks: Arc<Mutex<HashMap<u16, JoinHandle<()>>>>,
}

impl StreamManager {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 모든 활성 스트림을 중지하고 DB에서 다시 로드하여 시작 (서버 시작 시 사용)
    pub async fn reload_streams(&self) {
        // 1. 기존 작업 모두 중지
        {
            let mut tasks = self.tasks.lock().unwrap();
            for (_, handle) in tasks.drain() {
                handle.abort();
            }
        }
        tracing::info!("🔄 Reloading streams...");

        // 2. DB에서 로드 후 시작
        match db::get_all_streams(&self.db_pool).await {
            Ok(streams) => {
                // 👇 [수정 1] streams의 소유권이 넘어가기 전에 개수를 먼저 저장
                let count = streams.len();

                for s in streams {
                    self.start_stream(
                        s.listen_port as u16,
                        &s.forward_host,
                        s.forward_port as u16,
                        &s.protocol,
                    )
                    .await;
                }
                // 여기서 count 사용
                tracing::info!("✅ Loaded {} streams", count);
            }
            Err(e) => tracing::error!("❌ Failed to load streams from DB: {}", e),
        }
    }

    /// 단일 스트림 시작
    pub async fn start_stream(
        &self,
        listen_port: u16,
        forward_host: &str,
        forward_port: u16,
        protocol: &str,
    ) {
        // 이미 실행 중인 포트라면 중지
        self.stop_stream(listen_port);

        let forward_addr = format!("{}:{}", forward_host, forward_port);
        let protocol = protocol.to_lowercase();
        let port_clone = listen_port;
        let fwd_clone = forward_addr.clone();

        tracing::info!(
            "▶️ Starting {} Stream: :{} -> {}",
            protocol.to_uppercase(),
            listen_port,
            forward_addr
        );

        let handle = if protocol == "udp" {
            tokio::spawn(async move {
                if let Err(e) = run_udp_proxy(port_clone, &fwd_clone).await {
                    tracing::error!("UDP Stream Error on {}: {}", port_clone, e);
                }
            })
        } else {
            // Default TCP
            tokio::spawn(async move {
                if let Err(e) = run_tcp_proxy(port_clone, &fwd_clone).await {
                    tracing::error!("TCP Stream Error on {}: {}", port_clone, e);
                }
            })
        };

        self.tasks.lock().unwrap().insert(listen_port, handle);
    }

    /// 단일 스트림 중지
    pub fn stop_stream(&self, listen_port: u16) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(handle) = tasks.remove(&listen_port) {
            handle.abort();
            tracing::info!("⏹️ Stopped Stream on port {}", listen_port);
        }
    }
}

/// TCP 프록시 구현 (양방향 Copy)
async fn run_tcp_proxy(listen_port: u16, forward_addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", listen_port)).await?;

    loop {
        let (mut inbound, client_addr) = listener.accept().await?;
        let target = forward_addr.to_string();

        tokio::spawn(async move {
            match TcpStream::connect(&target).await {
                Ok(mut outbound) => {
                    // 양방향 데이터 전송 (Zero Copy)
                    let res = timeout(
                        Duration::from_secs(constants::timeout::TCP_TIMEOUT_SECS),
                        tokio::io::copy_bidirectional(&mut inbound, &mut outbound),
                    )
                    .await;

                    match res {
                        Ok(Ok(_)) => {
                            tracing::debug!("TCP connection closed gracefully ({})", client_addr);
                        }
                        Ok(Err(e)) => {
                            tracing::debug!("TCP connection closed ({}: {})", client_addr, e);
                        }
                        Err(_) => {
                            tracing::debug!("TCP connection timed out ({})", client_addr);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to connect to upstream {}: {}", target, e);
                }
            }
        });
    }
}

/// UDP 프록시 구현 (NAT 테이블 방식)
async fn run_udp_proxy(listen_port: u16, forward_addr: &str) -> std::io::Result<()> {
    // 1. 리스너 소켓 바인딩
    let listener = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", listen_port)).await?);

    // 2. 클라이언트 세션 관리 (Client Addr -> Upstream Socket)
    let sessions: Arc<Mutex<HashMap<SocketAddr, Arc<UdpSocket>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut buf = [0u8; constants::network::UDP_BUFFER_SIZE]; // Max UDP packet size

    loop {
        // 클라이언트로부터 데이터 수신
        let (len, src_addr) = match listener.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("UDP recv error: {}", e);
                continue;
            }
        };

        let data = &buf[..len];

        // 👇 [수정 2] Lock 범위를 최소화하여 await 호출 시 Lock을 들고 있지 않게 함
        // 1) 먼저 세션이 있는지 확인 (Lock)
        let existing_socket = {
            let map = sessions.lock().unwrap();
            map.get(&src_addr).cloned()
        }; // 여기서 Lock 해제됨

        let upstream_socket = if let Some(socket) = existing_socket {
            socket
        } else {
            // 2) 없으면 새로 생성 (Async 작업 - Lock 없이 수행)
            // 새 클라이언트: 업스트림과 연결할 새 소켓 생성 (Ephemeral Port)
            let new_socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    tracing::error!("Failed to bind UDP upstream socket: {}", e);
                    continue;
                }
            };

            if let Err(e) = new_socket.connect(forward_addr).await {
                tracing::error!("Failed to connect UDP to {}: {}", forward_addr, e);
                continue;
            }

            // 3) 다시 Lock을 걸고 저장 (중복 생성 방지 체크 포함)
            let mut map = sessions.lock().unwrap();
            // 그 사이에 다른 스레드가 만들었을 수도 있으니 다시 체크
            if let Some(s) = map.get(&src_addr) {
                s.clone()
            } else {
                map.insert(src_addr, new_socket.clone());

                // 🔄 [응답 처리 루프] 업스트림 -> 클라이언트
                let listener_clone = listener.clone();
                let upstream_clone = new_socket.clone();
                let src_addr_clone = src_addr;
                let sessions_clone = sessions.clone();

                tokio::spawn(async move {
                    let mut resp_buf = [0u8; constants::network::UDP_BUFFER_SIZE];
                    loop {
                        // 1분간 응답 없으면 세션 종료 (메모리 누수 방지)
                        match timeout(
                            Duration::from_secs(constants::timeout::UDP_SESSION_TIMEOUT_SECS),
                            upstream_clone.recv(&mut resp_buf),
                        )
                        .await
                        {
                            Ok(Ok(n)) => {
                                // 받은 데이터를 원본 클라이언트에게 전송
                                if let Err(e) =
                                    listener_clone.send_to(&resp_buf[..n], src_addr_clone).await
                                {
                                    tracing::debug!("Failed to send UDP back to client: {}", e);
                                    break;
                                }
                            }
                            _ => {
                                // Timeout or Error: 세션 정리
                                tracing::debug!("UDP session timed out for {}", src_addr_clone);
                                sessions_clone.lock().unwrap().remove(&src_addr_clone);
                                break;
                            }
                        }
                    }
                });

                new_socket
            }
        };

        // 업스트림으로 데이터 전송
        if let Err(e) = upstream_socket.send(data).await {
            tracing::error!("Failed to forward UDP packet: {}", e);
        }
    }
}
