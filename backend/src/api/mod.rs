use crate::acme::AcmeManager;
use crate::auth::{self, AuthError, Claims};
use crate::db::{self, DbPool, TrafficStatRow};
use crate::state::{AppState, ProxyConfig, HostConfig, LocationConfig};
use axum::{
    extract::{State, Json, Query, Path as AxumPath},
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
    pub locations: Vec<LocationConfig>,
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
    pub hours: Option<i64>, // Default 24
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub lines: Option<usize>,
}

pub fn router(app_state: Arc<AppState>, db_pool: DbPool, prometheus_handle: PrometheusHandle) -> Router {
    let state = ApiState { app_state, db_pool, prometheus_handle };

    let serve_dir = ServeDir::new("static")
        .not_found_service(ServeFile::new("static/index.html"));

    // 인증이 필요한 API 라우터
    let protected_api = Router::new()
        .route("/hosts", get(list_hosts).post(add_host))
        .route("/hosts/{domain}", delete(delete_host_handler))
        .route("/hosts/{domain}/locations", post(add_location).delete(delete_location_handler))
        .route("/certs", post(request_cert))
        .route("/stats/realtime", get(get_realtime_stats))
        .route("/stats/history", get(get_history_stats))
        .route("/logs", get(get_logs));

    // 전체 라우터 조립
    Router::new()
        .route("/api/login", post(login_handler))
        .route("/metrics", get(metrics_handler)) // Public metrics endpoint
        .nest("/api", protected_api)
        .fallback_service(serve_dir) // axum 0.8: nest_service("/", ...) 대신 fallback_service 사용
        .layer(CorsLayer::permissive()) 
        .with_state(state)
}

async fn metrics_handler(State(state): State<ApiState>) -> String {
    state.prometheus_handle.render()
}

/// 로그인 핸들러
async fn login_handler(
    State(state): State<ApiState>,
    Json(payload): Json<LoginReq>,
) -> Result<Json<LoginRes>, AuthError> {
    // 1. 사용자 조회
    let user = db::get_user(&state.db_pool, &payload.username)
        .await
        .map_err(|_| AuthError::InternalServerError)?;

    let (_, hash) = user.ok_or(AuthError::WrongCredentials)?;

    // 2. 비밀번호 검증
    if auth::verify_password(&payload.password, &hash) {
        // 3. JWT 발급
        let token = auth::create_jwt(&payload.username)?;
        Ok(Json(LoginRes { token }))
    } else {
        Err(AuthError::WrongCredentials)
    }
}

/// 헬퍼: DB 내용을 읽어 메모리(AppState)에 반영
async fn sync_state(state: &ApiState) {
    let hosts_result = db::get_all_hosts(&state.db_pool).await;
    let locations_result = db::get_all_locations(&state.db_pool).await;

    if let (Ok(rows), Ok(loc_rows)) = (hosts_result, locations_result) {
        // Location 정보를 host_id 별로 그룹화
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
        state.app_state.update_config(ProxyConfig { hosts });
        tracing::info!("♻️ State synced with DB (Hosts & Locations)");
    } else {
        tracing::error!("❌ Failed to sync state from DB");
    }
}

// --- Protected Handlers (Claims 인자 추가) ---

async fn list_hosts(
    _: Claims, // 인증 요구
    State(state): State<ApiState>,
) -> Json<Vec<HostRes>> {
    let hosts = state.app_state.config.load();
    let res: Vec<HostRes> = hosts.hosts.iter()
        .map(|(d, c)| HostRes { 
            domain: d.clone(), 
            target: c.target.clone(),
            scheme: c.scheme.clone(),
            locations: c.locations.clone(),
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
    
    if let Err(e) = db::upsert_host(&state.db_pool, &payload.domain, &payload.target, &scheme).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
    sync_state(&state).await;
    StatusCode::CREATED
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

// Location Handlers

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

// --- Stats APIs ---

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
    
    // 오늘 날짜의 로그 파일 찾기 (access.log.YYYY-MM-DD)
    // chrono를 사용하여 오늘 날짜 포맷팅
    let now = chrono::Local::now();
    let filename = format!("access.log.{}", now.format("%Y-%m-%d"));
    let path = log_dir.join(filename);

    if !path.exists() {
        return Json(vec!["No logs found for today.".to_string()]);
    }

    match fs::read_to_string(path) {
        Ok(content) => {
            let lines: Vec<String> = content
                .lines()
                .rev() // 최신순
                .take(limit)
                .map(|s| s.to_string())
                .collect();
            Json(lines)
        }
        Err(e) => Json(vec![format!("Failed to read log file: {}", e)]),
    }
}

// AuthError에 InternalServerError 변형 추가 필요 (간단히 처리)
impl From<sqlx::Error> for AuthError {
    fn from(_: sqlx::Error) -> Self {
        AuthError::TokenCreation // 임시로 500 에러 매핑
    }
}