use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    pub path: String,
    pub target: String,
    pub scheme: String,
    #[serde(default)]
    pub rewrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub id: i64,
    pub target: String,
    pub scheme: String, // "http" or "https"
    #[serde(default)]
    pub locations: Vec<LocationConfig>,
    #[serde(default)]
    pub ssl_forced: bool,
    pub redirect_to: Option<String>,
    #[serde(default = "default_redirect_status")]
    pub redirect_status: u16,
    pub access_list_id: Option<i64>,
}

fn default_redirect_status() -> u16 {
    301
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
    pub name: String,
    pub value: String,
    pub target: String, // request/response
}

/// 프록시 라우팅 설정
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 도메인 -> 설정
    pub hosts: HashMap<String, HostConfig>,
    /// Access List ID -> Config
    #[serde(skip)]
    pub access_lists: HashMap<i64, AccessListConfig>,
    /// Host ID -> Headers
    #[serde(skip)]
    pub headers: HashMap<i64, Vec<HeaderConfig>>,
}

/// 실시간 메트릭 (Atomic Counters)
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

/// 전역 상태 관리 (Thread-Safe)
#[derive(Clone)]
pub struct AppState {
    /// 설정에 대한 Atomic 포인터. 읽기는 Lock-Free.
    pub config: Arc<ArcSwap<ProxyConfig>>,

    /// 메트릭 통계
    pub metrics: Arc<Metrics>,

    /// 에러 페이지 템플릿 (Atomic Update 지원)
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

    // Note: Host ID is not directly available from domain lookup in `get_host_config` currently without reverse lookup or changing HostConfig.
    // For now, we will assume `proxy.rs` will look up headers if we expose them.
    // But `ProxyConfig` maps Domain -> HostConfig. We need Host ID to look up headers in `headers` map.
    // Let's add `id` to `HostConfig` struct as well to make this easier.
    
    /// 설정을 통째로 교체합니다. (Atomic)
    pub fn update_config(&self, new_config: ProxyConfig) {
        self.config.store(Arc::new(new_config));
    }
    
    /// 에러 템플릿 업데이트
    pub fn update_error_template(&self, new_template: String) {
        self.error_template.store(Arc::new(new_template));
    }

    pub fn get_headers(&self, host_id: i64) -> Vec<HeaderConfig> {
        let config = self.config.load();
        // headers 맵에서 host_id로 조회, 없으면 빈 벡터 반환
        config.headers.get(&host_id).cloned().unwrap_or_default()
    }
}
