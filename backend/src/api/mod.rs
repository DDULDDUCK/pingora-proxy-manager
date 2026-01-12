pub mod handlers;
pub mod types;

use crate::db::DbPool;
use crate::state::AppState;
use crate::stream_manager::StreamManager;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

// Handlers
use handlers::access_lists::*;
use handlers::auth::*;
use handlers::certs::*;
use handlers::hosts::*;
use handlers::stats::*;
use handlers::streams::*;
use handlers::users::*;

#[derive(Clone)]
pub struct ApiState {
    pub app_state: Arc<AppState>,
    pub db_pool: DbPool,
    pub prometheus_handle: PrometheusHandle,
    pub stream_manager: Arc<StreamManager>,
}

pub fn router(
    app_state: Arc<AppState>,
    db_pool: DbPool,
    prometheus_handle: PrometheusHandle,
    stream_manager: Arc<StreamManager>,
) -> Router {
    let state = ApiState {
        app_state,
        db_pool,
        prometheus_handle,
        stream_manager,
    };

    let serve_dir = ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));

    // 인증이 필요한 API 라우터
    let protected_api = Router::new()
        // Hosts
        .route("/hosts", get(list_hosts).post(add_host))
        .route("/hosts/{domain}", delete(delete_host_handler))
        .route(
            "/hosts/{domain}/locations",
            post(add_location).delete(delete_location_handler),
        )
        .route(
            "/hosts/{domain}/headers",
            get(list_host_headers).post(add_header_to_host),
        )
        .route(
            "/hosts/{domain}/headers/{header_id}",
            delete(delete_host_header),
        )
        // Certs
        .route("/certs", get(list_certs).post(request_cert))
        .route("/certs/upload", post(upload_cert))
        // Stats & Logs
        .route("/stats/realtime", get(get_realtime_stats))
        .route("/stats/history", get(get_history_stats))
        .route("/logs", get(get_logs))
        // Streams
        .route("/streams", get(list_streams).post(add_stream))
        .route("/streams/{port}", delete(delete_stream_handler))
        // Settings
        .route(
            "/settings/error-page",
            get(get_error_page).post(update_error_page),
        )
        // Access Lists
        .route(
            "/access-lists",
            get(list_access_lists).post(create_access_list),
        )
        .route("/access-lists/{id}", delete(delete_access_list_handler))
        .route(
            "/access-lists/{id}/clients",
            post(add_access_list_client_handler),
        )
        .route(
            "/access-lists/{id}/clients/{username}",
            delete(delete_access_list_client_handler),
        )
        .route("/access-lists/{id}/ips", post(add_access_list_ip_handler))
        .route(
            "/access-lists/{id}/ips/{ip}",
            delete(delete_access_list_ip_handler),
        )
        // DNS Providers
        .route(
            "/dns-providers",
            get(list_dns_providers).post(create_dns_provider_handler),
        )
        .route("/dns-providers/{id}", delete(delete_dns_provider_handler))
        // User Management (Admin only)
        .route("/users", get(list_users).post(create_user_handler))
        .route(
            "/users/{id}",
            put(update_user_handler).delete(delete_user_handler),
        )
        .route("/users/me", get(get_current_user))
        .route("/users/me/password", put(change_own_password))
        // Audit Logs
        .route("/audit-logs", get(get_audit_logs_handler));

    // 전체 라우터 조립
    Router::new()
        .route("/api/login", post(login_handler))
        .route("/metrics", get(metrics_handler))
        .nest("/api", protected_api)
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// Shared helper function used by handlers
pub(crate) async fn sync_state(state: &ApiState) {
    match crate::config::loader::ConfigLoader::load_from_db(&state.db_pool).await {
        Ok(config) => {
            state.app_state.update_config(config);
            tracing::info!("♻️ State synced with DB");
        }
        Err(e) => {
            tracing::error!("❌ Failed to sync state from DB: {}", e);
        }
    }
}
