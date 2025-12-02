mod acme;
mod api;
mod auth;
mod db;
mod proxy;
mod state;

use crate::proxy::DynamicProxy;
use crate::state::{AppState, ProxyConfig, HostConfig};
use pingora::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    // 1. 로깅 초기화
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Pingora Proxy Manager...");

    // Tokio 런타임 시작 (Pingora는 자체 런타임을 가질 수 있지만, Axum 실행을 위해 필요)
    // Pingora의 Server::run_forever()는 블로킹이므로, Axum은 별도 스레드나 Pingora 런타임 전에 띄워야 함.
    // 여기서는 Pingora가 메인 스레드를 점유하므로, Axum을 별도 스레드(Tokio Runtime)에서 실행합니다.

    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // 상태 공유 객체
    let state = Arc::new(AppState::new());
    let state_for_api = state.clone();

    rt.block_on(async move {
        // 2. DB 초기화
        let db_url = "sqlite:data.db?mode=rwc";
        let pool = db::init_db(db_url).await.expect("Failed to init DB");
        
        // 초기 관리자 계정 생성 (없으면)
        let admin_exists = db::get_user(&pool, "admin").await.unwrap().is_some();
        if !admin_exists {
            let hash = auth::hash_password("changeme").unwrap();
            db::create_user(&pool, "admin", &hash).await.unwrap();
            tracing::info!("👤 Created default admin user: admin / changeme");
        }

        // 3. 초기 상태 로드
        let rows = db::get_all_hosts(&pool).await.unwrap_or_default();
        let mut hosts = HashMap::new();
        for row in rows {
            hosts.insert(row.domain, HostConfig {
                target: row.target,
                scheme: row.scheme,
            });
        }
        state_for_api.update_config(ProxyConfig { hosts });
        tracing::info!("✅ Initial configuration loaded from DB");

        // 4. API 서버 실행 (81번 포트)
        tokio::spawn(async move {
            let app = api::router(state_for_api, pool);
            let listener = tokio::net::TcpListener::bind("0.0.0.0:81").await.unwrap();
            tracing::info!("🎮 Control Plane (API) running on port 81");
            axum::serve(listener, app).await.unwrap();
        });
    });

    // 5. Pingora 서버 실행 (메인 스레드 점유)
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let mut my_proxy = http_proxy_service(
        &my_server.configuration,
        DynamicProxy {
            state: state.clone(), // API가 업데이트하는 그 state를 공유
        },
    );

    my_proxy.add_tcp("0.0.0.0:8080");

    my_server.add_service(my_proxy);
    tracing::info!("🚀 Data Plane (Proxy) running on port 8080");
    my_server.run_forever();
}
