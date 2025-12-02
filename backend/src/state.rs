use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub target: String,
    pub scheme: String, // "http" or "https"
}

/// 프록시 라우팅 설정
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 도메인 -> 설정
    pub hosts: HashMap<String, HostConfig>,
}

/// 전역 상태 관리 (Thread-Safe)
#[derive(Clone)]
pub struct AppState {
    /// 설정에 대한 Atomic 포인터. 읽기는 Lock-Free.
    pub config: Arc<ArcSwap<ProxyConfig>>,
    
    /// ACME Challenge 토큰 저장소 (Token -> Key Authorization)
    pub acme_challenges: Arc<RwLock<HashMap<String, String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(ArcSwap::from_pointee(ProxyConfig::default())),
            acme_challenges: Arc::new(RwLock::new(HashMap::new())),
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
