use crate::acme::AcmeManager;
use crate::auth::{self, AuthError, Claims};
use crate::db::{self, DbPool, TrafficStatRow};
use crate::state::{AppState, ProxyConfig, HostConfig, LocationConfig};
use crate::stream_manager::StreamManager;
use axum::{
    extract::{State, Json, Query, Path as AxumPath, Multipart},
    routing::{get, post, delete},
    Router, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use std::fs;
use std::path::Path;
use metrics_exporter_prometheus::PrometheusHandle;

#[derive(Clone)]
pub struct ApiState {
    pub app_state: Arc<AppState>,
    pub db_pool: DbPool,
    pub prometheus_handle: PrometheusHandle,
    pub stream_manager: Arc<StreamManager>,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginRes {
    pub token: String,
}

#[derive(Deserialize)]
pub struct CreateHostReq {
    pub domain: String,
    pub target: String,
    pub scheme: Option<String>, 
    pub ssl_forced: Option<bool>,
    pub redirect_to: Option<String>,
    pub redirect_status: Option<i64>,
    pub access_list_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateLocationReq {
    pub path: String,
    pub target: String,
    pub scheme: Option<String>,
    pub rewrite: Option<bool>,
}

#[derive(Deserialize)]
pub struct DeleteLocationQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct HostRes {
    pub domain: String,
    pub target: String,
    pub scheme: String,
    pub ssl_forced: bool,
    pub redirect_to: Option<String>,
    pub redirect_status: u16,
    pub locations: Vec<LocationConfig>,
    pub access_list_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateCertReq {
    pub domain: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct RealtimeStatsRes {
    pub requests: u64,
    pub bytes: u64,
    pub status_2xx: u64,
    pub status_4xx: u64,
    pub status_5xx: u64,
}

#[derive(Deserialize)]
pub struct HistoryStatsQuery {
    pub hours: Option<i64>, 
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub lines: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateStreamReq {
    pub listen_port: u16,
    pub forward_host: String,
    pub forward_port: u16,
    pub protocol: Option<String>, 
}

#[derive(Serialize)]
pub struct StreamRes {
    pub id: i64,
    pub listen_port: i64,
    pub forward_host: String,
    pub forward_port: i64,
    pub protocol: String,
}

#[derive(Deserialize)]
pub struct ErrorPageReq {
    pub html: String,
}

// --- Access List Structs ---

#[derive(Deserialize)]
pub struct CreateAccessListReq {
    pub name: String,
    #[serde(default)]
    pub clients: Vec<AccessListClientReq>,
    #[serde(default)]
    pub ips: Vec<AccessListIpReq>,
}

#[derive(Deserialize)]
pub struct AccessListClientReq {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct AccessListIpReq {
    pub ip: String,
    pub action: String, // "allow" or "deny"
}

#[derive(Serialize)]
pub struct AccessListRes {
    pub id: i64,
    pub name: String,
    pub clients: Vec<AccessListClientRes>,
    pub ips: Vec<AccessListIpRes>,
}

#[derive(Serialize)]
pub struct AccessListClientRes {
    pub username: String,
}

#[derive(Serialize)]
pub struct AccessListIpRes {
    pub ip: String,
    pub action: String,
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
        .route("/hosts", get(list_hosts).post(add_host))
        .route("/hosts/{domain}", delete(delete_host_handler)) 
        .route("/hosts/{domain}/locations", post(add_location).delete(delete_location_handler))
        .route("/certs", post(request_cert))
        .route("/certs/upload", post(upload_cert))
        .route("/stats/realtime", get(get_realtime_stats))
        .route("/stats/history", get(get_history_stats))
        .route("/logs", get(get_logs))
        .route("/streams", get(list_streams).post(add_stream))
        .route("/streams/{port}", delete(delete_stream_handler))
        .route("/settings/error-page", get(get_error_page).post(update_error_page))
        // Access Lists
        .route("/access-lists", get(list_access_lists).post(create_access_list))
        .route("/access-lists/{id}", delete(delete_access_list_handler))
        .route("/access-lists/{id}/clients", post(add_access_list_client_handler))
        .route("/access-lists/{id}/clients/{username}", delete(delete_access_list_client_handler))
        .route("/access-lists/{id}/ips", post(add_access_list_ip_handler))
        .route("/access-lists/{id}/ips/{ip}", delete(delete_access_list_ip_handler));

    // 전체 라우터 조립
    Router::new()
        .route("/api/login", post(login_handler))
        .route("/metrics", get(metrics_handler))
        .nest("/api", protected_api)
        .fallback_service(serve_dir) 
        .layer(CorsLayer::permissive()) 
        .with_state(state)
}

async fn metrics_handler(State(state): State<ApiState>) -> String {
    state.prometheus_handle.render()
}

async fn login_handler(
    State(state): State<ApiState>,
    Json(payload): Json<LoginReq>,
) -> Result<Json<LoginRes>, AuthError> {
    let user = db::get_user(&state.db_pool, &payload.username)
        .await
        .map_err(|_| AuthError::InternalServerError)?;

    let (_, hash) = user.ok_or(AuthError::WrongCredentials)?;

    if auth::verify_password(&payload.password, &hash) {
        let token = auth::create_jwt(&payload.username)?;
        Ok(Json(LoginRes { token }))
    } else {
        Err(AuthError::WrongCredentials)
    }
}

async fn sync_state(state: &ApiState) {
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

// --- Handlers ---

async fn list_hosts(
    _: Claims,
    State(state): State<ApiState>,
) -> Json<Vec<HostRes>> {
    let hosts = state.app_state.config.load();
    let res: Vec<HostRes> = hosts.hosts.iter()
        .map(|(d, c)| HostRes { 
            domain: d.clone(), 
            target: c.target.clone(),
            scheme: c.scheme.clone(),
            ssl_forced: c.ssl_forced,
            redirect_to: c.redirect_to.clone(),
            redirect_status: c.redirect_status,
            locations: c.locations.clone(),
            access_list_id: c.access_list_id,
        })
        .collect();
    Json(res)
}

async fn add_host(
    _: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateHostReq>,
) -> StatusCode {
    let scheme = payload.scheme.unwrap_or_else(|| "http".to_string());
    let ssl_forced = payload.ssl_forced.unwrap_or(false);
    let redirect_status = payload.redirect_status.unwrap_or(301);
    
    match db::upsert_host(
        &state.db_pool, 
        &payload.domain, 
        &payload.target, 
        &scheme, 
        ssl_forced,
        payload.redirect_to,
        redirect_status,
        payload.access_list_id,
    ).await {
        Ok(_) => {
            sync_state(&state).await;
            StatusCode::CREATED
        }
        Err(e) => {
            tracing::error!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn delete_host_handler(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
) -> StatusCode {
    if let Err(e) = db::delete_host(&state.db_pool, &domain).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sync_state(&state).await;
    StatusCode::OK
}

async fn add_location(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
    Json(payload): Json<CreateLocationReq>,
) -> StatusCode {
    let host_id = match db::get_host_id(&state.db_pool, &domain).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let scheme = payload.scheme.unwrap_or_else(|| "http".to_string());
    let rewrite = payload.rewrite.unwrap_or(false);

    if let Err(e) = db::upsert_location(&state.db_pool, host_id, &payload.path, &payload.target, &scheme, rewrite).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sync_state(&state).await;
    StatusCode::CREATED
}

async fn delete_location_handler(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
    Query(q): Query<DeleteLocationQuery>,
) -> StatusCode {
    let host_id = match db::get_host_id(&state.db_pool, &domain).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if let Err(e) = db::delete_location(&state.db_pool, host_id, &q.path).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sync_state(&state).await;
    StatusCode::OK
}

async fn request_cert(
    _: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateCertReq>,
) -> StatusCode {
    let manager = AcmeManager::new(state.app_state.clone(), state.db_pool.clone(), payload.email);
    tokio::spawn(async move {
        if let Err(e) = manager.request_certificate(&payload.domain).await {
            tracing::error!("❌ Failed to issue cert for {}: {}", payload.domain, e);
        }
    });
    StatusCode::ACCEPTED
}

async fn upload_cert(
    _: Claims,
    State(state): State<ApiState>,
    mut multipart: Multipart,
) -> StatusCode {
    let mut cert_data = None;
    let mut key_data = None;
    let mut domain = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "domain" {
            if let Ok(txt) = field.text().await { domain = Some(txt); }
        } else if name == "cert" {
            if let Ok(bytes) = field.bytes().await { cert_data = Some(bytes); }
        } else if name == "key" {
            if let Ok(bytes) = field.bytes().await { key_data = Some(bytes); }
        }
    }

    if let (Some(d), Some(c), Some(k)) = (domain, cert_data, key_data) {
        let cert_dir = Path::new("data/certs/custom");
        if !cert_dir.exists() { let _ = fs::create_dir_all(cert_dir); }
        let cert_path = cert_dir.join(format!("{}.crt", d));
        let key_path = cert_dir.join(format!("{}.key", d));
        if fs::write(&cert_path, c).is_err() || fs::write(&key_path, k).is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        tracing::info!("💾 Custom certificate uploaded for {}", d);
        return StatusCode::CREATED;
    }
    StatusCode::BAD_REQUEST
}

async fn get_realtime_stats(
    _: Claims,
    State(state): State<ApiState>,
) -> Json<RealtimeStatsRes> {
    let m = &state.app_state.metrics;
    Json(RealtimeStatsRes {
        requests: m.total_requests.load(Ordering::Relaxed),
        bytes: m.total_bytes.load(Ordering::Relaxed),
        status_2xx: m.status_2xx.load(Ordering::Relaxed),
        status_4xx: m.status_4xx.load(Ordering::Relaxed),
        status_5xx: m.status_5xx.load(Ordering::Relaxed),
    })
}

async fn get_history_stats(
    _: Claims,
    State(state): State<ApiState>,
    Query(q): Query<HistoryStatsQuery>,
) -> Json<Vec<TrafficStatRow>> {
    let hours = q.hours.unwrap_or(24);
    let end_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let start_ts = end_ts - (hours * 3600);
    match db::get_traffic_stats(&state.db_pool, start_ts, end_ts).await {
        Ok(rows) => Json(rows),
        Err(e) => {
            tracing::error!("DB Stats Error: {}", e);
            Json(vec![])
        }
    }
}

async fn get_logs(
    _: Claims,
    Query(q): Query<LogsQuery>,
) -> Json<Vec<String>> {
    let limit = q.lines.unwrap_or(100);
    let log_dir = Path::new("logs");
    let now = chrono::Local::now();
    let filename = format!("access.log.{}", now.format("%Y-%m-%d"));
    let path = log_dir.join(filename);
    if !path.exists() { return Json(vec!["No logs found for today.".to_string()]); }
    match fs::read_to_string(path) {
        Ok(content) => {
            let lines: Vec<String> = content.lines().rev().take(limit).map(|s| s.to_string()).collect();
            Json(lines)
        }
        Err(e) => Json(vec![format!("Failed to read log file: {}", e)]),
    }
}

async fn list_streams(
    _: Claims,
    State(state): State<ApiState>,
) -> Json<Vec<StreamRes>> {
    match db::get_all_streams(&state.db_pool).await {
        Ok(rows) => Json(rows.into_iter().map(|r| StreamRes {
            id: r.id,
            listen_port: r.listen_port,
            forward_host: r.forward_host,
            forward_port: r.forward_port,
            protocol: r.protocol,
        }).collect()),
        Err(e) => {
            tracing::error!("Failed to fetch streams: {}", e);
            Json(vec![])
        }
    }
}

async fn add_stream(
    _: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateStreamReq>,
) -> StatusCode {
    let protocol = payload.protocol.unwrap_or_else(|| "tcp".to_string());
    if let Err(e) = db::upsert_stream(&state.db_pool, payload.listen_port as i64, &payload.forward_host, payload.forward_port as i64, &protocol).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    state.stream_manager.start_stream(payload.listen_port, &payload.forward_host, payload.forward_port, &protocol).await;
    StatusCode::CREATED
}

async fn delete_stream_handler(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath(port): AxumPath<u16>,
) -> StatusCode {
    if let Err(e) = db::delete_stream(&state.db_pool, port as i64).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    state.stream_manager.stop_stream(port);
    StatusCode::OK
}

async fn get_error_page(_: Claims) -> String {
    fs::read_to_string("data/templates/error.html").unwrap_or_default()
}

async fn update_error_page(
    _: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<ErrorPageReq>,
) -> StatusCode {
    if let Err(e) = fs::write("data/templates/error.html", &payload.html) {
        tracing::error!("Failed to write error template: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    state.app_state.update_error_template(payload.html);
    StatusCode::OK
}

// --- Access List Handlers ---

async fn list_access_lists(
    _: Claims,
    State(state): State<ApiState>,
) -> Json<Vec<AccessListRes>> {
    let al_rows = match db::get_all_access_lists(&state.db_pool).await {
        Ok(rows) => rows,
        Err(_) => return Json(vec![]),
    };

    let client_rows = db::get_access_list_clients(&state.db_pool).await.unwrap_or_default();
    let ip_rows = db::get_access_list_ips(&state.db_pool).await.unwrap_or_default();
    let mut res = Vec::new();

    for al in al_rows {
        let clients = client_rows.iter()
            .filter(|c| c.list_id == al.id)
            .map(|c| AccessListClientRes { username: c.username.clone() })
            .collect();
            
        let ips = ip_rows.iter()
            .filter(|i| i.list_id == al.id)
            .map(|i| AccessListIpRes { ip: i.ip_address.clone(), action: i.action.clone() })
            .collect();

        res.push(AccessListRes { id: al.id, name: al.name, clients, ips });
    }
    Json(res)
}

async fn create_access_list(
    _: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateAccessListReq>,
) -> StatusCode {
    let list_id = match db::create_access_list(&state.db_pool, &payload.name).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create access list: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    for client in payload.clients {
        if let Ok(hash) = auth::hash_password(&client.password) {
             let _ = db::add_access_list_client(&state.db_pool, list_id, &client.username, &hash).await;
        }
    }
    for ip in payload.ips {
        let _ = db::add_access_list_ip(&state.db_pool, list_id, &ip.ip, &ip.action).await;
    }
    sync_state(&state).await;
    StatusCode::CREATED
}

async fn delete_access_list_handler(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath(id): AxumPath<i64>,
) -> StatusCode {
    if let Err(e) = db::delete_access_list(&state.db_pool, id).await {
         tracing::error!("Failed to delete access list: {}", e);
         return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sync_state(&state).await;
    StatusCode::OK
}

// 👇 [추가됨] Client/IP 관리 핸들러들

async fn add_access_list_client_handler(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath(id): AxumPath<i64>,
    Json(payload): Json<AccessListClientReq>,
) -> StatusCode {
    let hash = match auth::hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if let Err(e) = db::add_access_list_client(&state.db_pool, id, &payload.username, &hash).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sync_state(&state).await;
    StatusCode::CREATED
}

async fn delete_access_list_client_handler(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath((id, username)): AxumPath<(i64, String)>,
) -> StatusCode {
    if let Err(e) = db::remove_access_list_client(&state.db_pool, id, &username).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sync_state(&state).await;
    StatusCode::OK
}

async fn add_access_list_ip_handler(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath(id): AxumPath<i64>,
    Json(payload): Json<AccessListIpReq>,
) -> StatusCode {
    if let Err(e) = db::add_access_list_ip(&state.db_pool, id, &payload.ip, &payload.action).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sync_state(&state).await;
    StatusCode::CREATED
}

async fn delete_access_list_ip_handler(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath((id, ip)): AxumPath<(i64, String)>,
) -> StatusCode {
    if let Err(e) = db::remove_access_list_ip(&state.db_pool, id, &ip).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    sync_state(&state).await;
    StatusCode::OK
}

impl From<sqlx::Error> for AuthError {
    fn from(_: sqlx::Error) -> Self {
        AuthError::TokenCreation 
    }
}