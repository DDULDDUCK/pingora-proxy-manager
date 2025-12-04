mod acme;
mod api;
mod auth;
mod db;
mod proxy;
mod state;

use crate::proxy::DynamicProxy;
use crate::state::{AppState, ProxyConfig, HostConfig, LocationConfig};
use pingora::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use metrics_exporter_prometheus::PrometheusBuilder;

fn main() {
    // 0. .env 파일 로드 (가장 먼저 실행)
    dotenvy::dotenv().ok();

    // 1. 로깅 초기화 (File + Stdout)
    let file_appender = tracing_appender::rolling::daily("logs", "access.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .json() // 파일에는 JSON으로 저장 (구조화된 로그)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
        )
        .init();

    tracing::info!("Starting Pingora Proxy Manager...");

    // 메트릭 레코더 초기화
    let recorder_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    // Tokio 런타임 시작
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // 상태 공유 객체
    let state = Arc::new(AppState::new());
    
    // 초기화용 state 복제 (메인 state는 아래 Pingora Proxy에서 사용)
    let state_for_init = state.clone();

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
        let hosts_result = db::get_all_hosts(&pool).await;
        let locations_result = db::get_all_locations(&pool).await;

        if let (Ok(rows), Ok(loc_rows)) = (hosts_result, locations_result) {
            let mut locations_map: HashMap<i64, Vec<LocationConfig>> = HashMap::new();
            for loc in loc_rows {
                locations_map.entry(loc.host_id).or_default().push(LocationConfig {
                    path: loc.path,
                    target: loc.target,
                    scheme: loc.scheme,
                    rewrite: loc.rewrite,
                });
            }

            let mut hosts = HashMap::new();
            for row in rows {
                let locs = locations_map.remove(&row.id).unwrap_or_default();
                hosts.insert(row.domain, HostConfig {
                    target: row.target,
                    scheme: row.scheme,
                    locations: locs,
                });
            }
            state_for_init.update_config(ProxyConfig { hosts });
            tracing::info!("✅ Initial configuration loaded from DB (Hosts & Locations)");
        } else {
            tracing::warn!("⚠️ Failed to load initial configuration from DB");
        }

        // 4. API 서버 실행 (81번 포트)
        let pool_for_api = pool.clone();
        let state_for_api = state_for_init.clone();
        let recorder_handle_for_api = recorder_handle.clone();
        tokio::spawn(async move {
            let app = api::router(state_for_api, pool_for_api, recorder_handle_for_api);
            let listener = tokio::net::TcpListener::bind("0.0.0.0:81").await.unwrap();
            tracing::info!("🎮 Control Plane (API) running on port 81");
            axum::serve(listener, app).await.unwrap();
        });

        // 5. 자동 갱신 스케줄러 (매 1시간마다 체크)
        let pool_for_acme = pool.clone();
        let state_for_acme = state_for_init.clone();
        tokio::spawn(async move {
            let acme_manager = acme::AcmeManager::new(
                state_for_acme, 
                pool_for_acme.clone(), 
                "admin@example.com".to_string() 
            );
            
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                
                tracing::info!("⏰ Checking for expiring certificates...");
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                let renewal_threshold = now + 30 * 24 * 60 * 60; 

                match db::get_expiring_certs(&pool_for_acme, renewal_threshold).await {
                    Ok(domains) => {
                        for domain in domains {
                            tracing::info!("♻️ Renewing certificate for {}", domain);
                            if let Err(e) = acme_manager.request_certificate(&domain).await {
                                tracing::error!("❌ Failed to renew certificate for {}: {}", domain, e);
                            }
                        }
                    },
                    Err(e) => tracing::error!("❌ Failed to check expiring certificates: {}", e),
                }
            }
        });

        // 6. 메트릭 수집 스케줄러 (매 1분마다 DB 저장)
        let pool_for_stats = pool.clone();
        let state_for_stats = state_for_init.clone();
        tokio::spawn(async move {
            loop {
                // 1분 대기 (정각에 맞추는 로직은 아님)
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                
                let (reqs, bytes, s2xx, s4xx, s5xx) = state_for_stats.metrics.reset();
                
                // 데이터가 있을 때만 저장 (옵션)
                if reqs > 0 {
                     let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
                     if let Err(e) = db::insert_traffic_stat(&pool_for_stats, now, reqs, bytes, s2xx, s4xx, s5xx).await {
                         tracing::error!("❌ Failed to save traffic stats: {}", e);
                     } else {
                         tracing::debug!("📊 Traffic stats saved: {} reqs", reqs);
                     }
                }
            }
        });
    });

    // 7. Pingora 서버 실행 (메인 스레드 점유)
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
