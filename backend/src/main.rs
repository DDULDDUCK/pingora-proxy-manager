mod acme;
mod api;
mod auth;
mod bootstrap;
mod config;
mod constants;
mod db;
mod error;
mod proxy;
mod state;
mod stream_manager;
mod tls_manager;

use crate::proxy::DynamicProxy;
use crate::state::AppState;
use crate::stream_manager::StreamManager;
use crate::tls_manager::SharedCertManager;
use pingora::listeners::tls::TlsSettings;
use pingora::prelude::*;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. 작업 디렉토리 자동 보정 (backend 폴더에서 실행 시 상위로 이동)
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.ends_with("backend") {
            if let Err(e) = std::env::set_current_dir("..") {
                eprintln!("Warning: Failed to change directory to project root: {}", e);
            } else {
                eprintln!(
                    "Note: Changed working directory to project root to locate data and logs"
                );
            }
        }
    }

    // 0. .env 파일 로드 (가장 먼저 실행)
    dotenvy::dotenv().ok();

    // 1. 로깅 초기화 (File + Stdout)
    let _guard = bootstrap::logging::init_logging();

    tracing::info!("Starting Pingora Proxy Manager...");

    // 메트릭 레코더 초기화
    let recorder_handle = bootstrap::metrics::init_metrics()?;

    // Tokio 런타임 시작
    let rt = tokio::runtime::Runtime::new()?;

    // 상태 공유 객체
    let state = Arc::new(AppState::new());

    // 초기화용 state 복제 (메인 state는 아래 Pingora Proxy에서 사용)
    let state_for_init = state.clone();

    rt.block_on(async move {
        // 2. DB 초기화
        let db_url = "sqlite:data/data.db?mode=rwc";
        let pool = bootstrap::db::init_db(db_url).await?;

        // 3. 초기 상태 로드
        match crate::config::loader::ConfigLoader::load_from_db(&pool).await {
            Ok(config) => {
                state_for_init.update_config(config);
                tracing::info!("✅ Initial configuration loaded from DB");
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to load initial configuration from DB: {}", e);
            }
        }

        // Stream Manager 초기화
        let stream_manager = Arc::new(StreamManager::new(pool.clone()));
        stream_manager.reload_streams().await; // 초기 로드

        // 4. API 서버 실행 (81번 포트)
        let pool_for_api = pool.clone();
        let state_for_api = state_for_init.clone();
        let recorder_handle_for_api = recorder_handle.clone();
        let stream_manager_for_api = stream_manager.clone(); // API용 복제

        tokio::spawn(async move {
            let app = api::router(
                state_for_api,
                pool_for_api,
                recorder_handle_for_api,
                stream_manager_for_api,
            );
            let listener = tokio::net::TcpListener::bind(constants::network::API_PORT_STR)
                .await
                .unwrap();
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
                "admin@example.com".to_string(),
            );

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;

                tracing::info!("⏰ Checking for expiring certificates...");
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                let renewal_threshold = now + 30 * 24 * 60 * 60;

                match db::get_expiring_certs(&pool_for_acme, renewal_threshold).await {
                    Ok(certs) => {
                        for (domain, provider_id) in certs {
                            tracing::info!(
                                "♻️ Renewing certificate for {} (Provider: {:?})",
                                domain,
                                provider_id
                            );
                            if let Err(e) =
                                acme_manager.request_certificate(&domain, provider_id).await
                            {
                                tracing::error!(
                                    "❌ Failed to renew certificate for {}: {}",
                                    domain,
                                    e
                                );
                            }
                        }
                    }
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
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    if let Err(e) =
                        db::insert_traffic_stat(&pool_for_stats, now, reqs, bytes, s2xx, s4xx, s5xx)
                            .await
                    {
                        tracing::error!("❌ Failed to save traffic stats: {}", e);
                    } else {
                        tracing::debug!("📊 Traffic stats saved: {} reqs", reqs);
                    }
                }
            }
        });
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    // 7. Pingora 서버 실행 (메인 스레드 점유)
    let mut my_server = Server::new(None)?;
    my_server.bootstrap();

    let mut my_proxy = http_proxy_service(
        &my_server.configuration,
        DynamicProxy {
            state: state.clone(), // API가 업데이트하는 그 state를 공유
        },
    );

    my_proxy.add_tcp(constants::network::PROXY_PORT_STR);

    // SNI 기반 동적 인증서 선택 설정
    let cert_manager = match tls_manager::DynamicCertManager::new(
        "data/certs",
        "data/certs/default.crt",
        "data/certs/default.key",
    ) {
        Ok(manager) => {
            // 기존 인증서 사전 로드
            if let Err(e) = manager.preload_certs() {
                tracing::warn!("⚠️ Failed to preload certificates: {}", e);
            }
            Some(SharedCertManager::new(manager))
        }
        Err(e) => {
            tracing::warn!(
                "⚠️ Failed to initialize dynamic cert manager: {}. Using static default cert.",
                e
            );
            None
        }
    };

    if let Some(cert_manager) = cert_manager {
        // SNI 기반 동적 인증서 선택 사용
        let mut tls_settings = TlsSettings::with_callbacks(Box::new(cert_manager))
            .expect("Failed to create TLS settings with callbacks");

        tls_settings.enable_h2();

        my_proxy.add_tls_with_settings(constants::network::TLS_PORT_STR, None, tls_settings);
        tracing::info!("🔐 TLS with SNI-based dynamic certificate selection enabled");
    } else {
        // 폴백: 디폴트 인증서만 사용
        my_proxy.add_tls(
            constants::network::TLS_PORT_STR,
            "data/certs/default.crt",
            "data/certs/default.key",
        )?;
        tracing::info!("🔐 TLS with static default certificate enabled");
    }

    my_server.add_service(my_proxy);
    tracing::info!("🚀 Data Plane (Proxy) running on port 8080 (HTTP) and 443 (HTTPS)");
    my_server.run_forever();
    Ok(())
}
