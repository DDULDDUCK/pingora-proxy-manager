pub mod types;
pub mod handlers;

use crate::auth::AuthError;
use crate::db::{self, DbPool};
use crate::state::{AppState, ProxyConfig, HostConfig, LocationConfig};
use crate::stream_manager::StreamManager;
use axum::{
    routing::{get, post, delete, put},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use metrics_exporter_prometheus::PrometheusHandle;

// Handlers
use handlers::auth::*;
use handlers::hosts::*;
use handlers::certs::*;
use handlers::streams::*;
use handlers::users::*;
use handlers::access_lists::*;
use handlers::stats::*;

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
    stream_manager: Arc<StreamManager>
) -> Router {
    let state = ApiState { app_state, db_pool, prometheus_handle, stream_manager };

    let serve_dir = ServeDir::new("static")
        .not_found_service(ServeFile::new("static/index.html"));

    // 인증이 필요한 API 라우터
    let protected_api = Router::new()
        // Hosts
        .route("/hosts", get(list_hosts).post(add_host))
        .route("/hosts/{domain}", delete(delete_host_handler))
        .route("/hosts/{domain}/locations", post(add_location).delete(delete_location_handler))
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
        .route("/settings/error-page", get(get_error_page).post(update_error_page))
        // Access Lists
        .route("/access-lists", get(list_access_lists).post(create_access_list))
        .route("/access-lists/{id}", delete(delete_access_list_handler))
        .route("/access-lists/{id}/clients", post(add_access_list_client_handler))
        .route("/access-lists/{id}/clients/{username}", delete(delete_access_list_client_handler))
        .route("/access-lists/{id}/ips", post(add_access_list_ip_handler))
        .route("/access-lists/{id}/ips/{ip}", delete(delete_access_list_ip_handler))
        // DNS Providers
        .route("/dns-providers", get(list_dns_providers).post(create_dns_provider_handler))
        .route("/dns-providers/{id}", delete(delete_dns_provider_handler))
        // User Management (Admin only)
        .route("/users", get(list_users).post(create_user_handler))
        .route("/users/{id}", put(update_user_handler).delete(delete_user_handler))
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
    let hosts_result = db::get_all_hosts(&state.db_pool).await;
    let locations_result = db::get_all_locations(&state.db_pool).await;
    let access_lists_result = db::get_all_access_lists(&state.db_pool).await;
    let clients_result = db::get_access_list_clients(&state.db_pool).await;
    let ips_result = db::get_access_list_ips(&state.db_pool).await;
    let headers_result = db::get_all_headers(&state.db_pool).await;

    if let (Ok(rows), Ok(loc_rows), Ok(al_rows), Ok(client_rows), Ok(ip_rows), Ok(header_rows)) = (
        hosts_result, 
        locations_result, 
        access_lists_result, 
        clients_result, 
        ips_result, 
        headers_result
    ) {
        // Locations
        let mut locations_map: HashMap<i64, Vec<LocationConfig>> = HashMap::new();
        for loc in loc_rows {
            locations_map.entry(loc.host_id).or_default().push(LocationConfig {
                path: loc.path,
                target: loc.target,
                scheme: loc.scheme,
                rewrite: loc.rewrite,
            });
        }

        // Access Lists
        let mut access_lists = HashMap::new();
        let mut clients_map: HashMap<i64, Vec<crate::state::AccessListClientConfig>> = HashMap::new();
        for c in client_rows {
            clients_map.entry(c.list_id).or_default().push(crate::state::AccessListClientConfig {
                username: c.username,
                password_hash: c.password_hash,
            });
        }
        let mut ips_map: HashMap<i64, Vec<crate::state::AccessListIpConfig>> = HashMap::new();
        for ip in ip_rows {
            ips_map.entry(ip.list_id).or_default().push(crate::state::AccessListIpConfig {
                ip: ip.ip_address,
                action: ip.action,
            });
        }
        for al in al_rows {
            access_lists.insert(al.id, crate::state::AccessListConfig {
                id: al.id,
                name: al.name,
                clients: clients_map.remove(&al.id).unwrap_or_default(),
                ips: ips_map.remove(&al.id).unwrap_or_default(),
            });
        }

        // Headers
        let mut headers: HashMap<i64, Vec<crate::state::HeaderConfig>> = HashMap::new();
        for h in header_rows {
            headers.entry(h.host_id).or_default().push(crate::state::HeaderConfig {
                name: h.name,
                value: h.value,
                target: h.target,
            });
        }

        // Hosts
        let mut hosts = HashMap::new();
        for row in rows {
            let locs = locations_map.remove(&row.id).unwrap_or_default();
            hosts.insert(row.domain, HostConfig {
                id: row.id,
                target: row.target,
                scheme: row.scheme,
                locations: locs,
                ssl_forced: row.ssl_forced,
                redirect_to: row.redirect_to,
                redirect_status: row.redirect_status as u16,
                access_list_id: row.access_list_id,
            });
        }
        
        state.app_state.update_config(ProxyConfig { hosts, access_lists, headers });
        tracing::info!("♻️ State synced with DB");
    } else {
        tracing::error!("❌ Failed to sync state from DB");
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(_: sqlx::Error) -> Self {
        AuthError::TokenCreation 
    }
}
