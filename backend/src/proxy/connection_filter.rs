use async_trait::async_trait;
use pingora::listeners::ConnectionFilter;
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug)]
pub struct IpBlockConnectionFilter {
    blocked_ips: HashSet<IpAddr>,
}

impl IpBlockConnectionFilter {
    pub fn from_env() -> Self {
        let raw = std::env::var("PPM_BLOCKED_IPS")
            .or_else(|_| std::env::var("BLOCKED_IPS"))
            .unwrap_or_default();

        let blocked_ips = raw
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    trimmed.parse::<IpAddr>().ok()
                }
            })
            .collect::<HashSet<_>>();

        if !blocked_ips.is_empty() {
            tracing::info!(
                "Connection filter enabled with {} blocked IP(s)",
                blocked_ips.len()
            );
        }

        Self { blocked_ips }
    }
}

#[async_trait]
impl ConnectionFilter for IpBlockConnectionFilter {
    async fn should_accept(&self, addr: Option<&SocketAddr>) -> bool {
        match addr {
            Some(socket_addr) => !self.blocked_ips.contains(&socket_addr.ip()),
            None => true,
        }
    }
}
