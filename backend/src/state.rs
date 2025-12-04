use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};

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
    pub target: String,
    pub scheme: String, // "http" or "https"
    #[serde(default)]
    pub locations: Vec<LocationConfig>,
}

/// 프록시 라우팅 설정
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 도메인 -> 설정
    pub hosts: HashMap<String, HostConfig>,
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
    
    /// ACME Challenge 토큰 저장소 (Token -> Key Authorization)
    pub acme_challenges: Arc<RwLock<HashMap<String, String>>>,

    /// 메트릭 통계
    pub metrics: Arc<Metrics>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(ArcSwap::from_pointee(ProxyConfig::default())),
            acme_challenges: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(Metrics::new()),
        }
    }

    /// 특정 호스트에 대한 설정을 조회합니다.
    pub fn get_host_config(&self, host: &str) -> Option<HostConfig> {
        let config = self.config.load();
        config.hosts.get(host).cloned()
    }

    /// 설정을 통째로 교체합니다. (Atomic)
    pub fn update_config(&self, new_config: ProxyConfig) {
        self.config.store(Arc::new(new_config));
    }
    
    /// ACME Challenge 등록
    pub fn add_acme_challenge(&self, token: String, key_auth: String) {
        let mut map = self.acme_challenges.write().unwrap();
        map.insert(token, key_auth);
    }

    /// ACME Challenge 조회
    pub fn get_acme_challenge(&self, token: &str) -> Option<String> {
        let map = self.acme_challenges.read().unwrap();
        map.get(token).cloned()
    }
    
    /// ACME Challenge 삭제 (검증 완료 후)
    pub fn remove_acme_challenge(&self, token: &str) {
        let mut map = self.acme_challenges.write().unwrap();
        map.remove(token);
    }
}
