use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    pub path: String,
    // Changed: target -> targets for load balancing
    pub targets: Vec<String>,
    pub scheme: String,
    #[serde(default)]
    pub rewrite: bool,
    #[serde(default = "default_verify_ssl")]
    pub verify_ssl: bool,
    pub upstream_sni: Option<String>,
}

/// Configuration for a specific virtual host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub id: i64,
    // Changed: target -> targets for load balancing
    pub targets: Vec<String>,
    pub scheme: String, // "http" or "https"
    #[serde(default)]
    pub locations: Vec<LocationConfig>,
    #[serde(default)]
    pub ssl_forced: bool,
    #[serde(default = "default_verify_ssl")]
    pub verify_ssl: bool,
    pub upstream_sni: Option<String>,
    pub redirect_to: Option<String>,
    #[serde(default = "default_redirect_status")]
    pub redirect_status: u16,
    pub access_list_id: Option<i64>,
    #[serde(default)]
    pub headers: Vec<HeaderConfig>,
}

fn default_redirect_status() -> u16 {
    301
}

fn default_verify_ssl() -> bool {
    true
}

// --- New Config Structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListConfig {
    pub id: i64,
    pub name: String,
    pub clients: Vec<AccessListClientConfig>, // Basic Auth users
    pub ips: Vec<AccessListIpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListClientConfig {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListIpConfig {
    pub ip: String,
    pub action: String, // allow/deny
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderConfig {
    pub id: i64,
    pub name: String,
    pub value: String,
    pub target: String, // request/response
}

/// Proxy routing and security configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Domain name to host configuration mapping.
    pub hosts: HashMap<String, HostConfig>,
    /// Access List ID to configuration mapping.
    #[serde(skip)]
    pub access_lists: HashMap<i64, AccessListConfig>,
    /// Host ID to header configurations mapping.
    #[serde(skip)]
    pub headers: HashMap<i64, Vec<HeaderConfig>>,
}

/// Real-time traffic metrics using atomic counters.
#[derive(Debug, Default)]
pub struct Metrics {
    pub total_requests: AtomicU64,
    pub total_bytes: AtomicU64,
    pub status_2xx: AtomicU64,
    pub status_4xx: AtomicU64,
    pub status_5xx: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    // 현재 값을 읽고 카운터를 0으로 리셋 (플러시용)
    pub fn reset(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.total_requests.swap(0, Ordering::Relaxed),
            self.total_bytes.swap(0, Ordering::Relaxed),
            self.status_2xx.swap(0, Ordering::Relaxed),
            self.status_4xx.swap(0, Ordering::Relaxed),
            self.status_5xx.swap(0, Ordering::Relaxed),
        )
    }
}

/// Global application state shared across threads.
#[derive(Clone)]
pub struct AppState {
    /// Thread-safe, lock-free access to proxy configuration.
    pub config: Arc<ArcSwap<ProxyConfig>>,

    /// Real-time traffic statistics.
    pub metrics: Arc<Metrics>,

    /// HTML template for custom error pages.
    pub error_template: Arc<ArcSwap<String>>,
}

impl AppState {
    pub fn new() -> Self {
        let error_template_str = fs::read_to_string("data/templates/error.html")
            .unwrap_or_else(|_| "<h1>502 Bad Gateway</h1><p>Pingora Proxy Manager</p>".to_string());

        Self {
            config: Arc::new(ArcSwap::from_pointee(ProxyConfig::default())),
            metrics: Arc::new(Metrics::new()),
            error_template: Arc::new(ArcSwap::from_pointee(error_template_str)),
        }
    }

    /// 특정 호스트에 대한 설정을 조회합니다.
    pub fn get_host_config(&self, host: &str) -> Option<HostConfig> {
        let config = self.config.load();
        config.hosts.get(host).cloned()
    }

    pub fn get_access_list(&self, id: i64) -> Option<AccessListConfig> {
        let config = self.config.load();
        config.access_lists.get(&id).cloned()
    }

    /// 특정 호스트 ID에 대한 헤더 목록을 조회합니다.
    pub fn get_headers(&self, host_id: i64) -> Vec<HeaderConfig> {
        let config = self.config.load();
        config.headers.get(&host_id).cloned().unwrap_or_default()
    }

    /// 설정을 통째로 교체합니다. (Atomic)
    pub fn update_config(&self, new_config: ProxyConfig) {
        self.config.store(Arc::new(new_config));
    }

    /// 에러 템플릿 업데이트
    pub fn update_error_template(&self, new_template: String) {
        self.error_template.store(Arc::new(new_template));
    }
}
