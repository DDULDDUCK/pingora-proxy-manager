use crate::acme::AcmeManager;
use crate::auth::{self, AuthError, Claims};
use crate::db::{self, DbPool};
use crate::state::{AppState, ProxyConfig, HostConfig};
use axum::{
    extract::{State, Json},
    routing::{get, post, delete},
    Router, http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

#[derive(Clone)]
pub struct ApiState {
    pub app_state: Arc<AppState>,
    pub db_pool: DbPool,
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

#[derive(Serialize)]
pub struct HostRes {
    pub domain: String,
    pub target: String,
    pub scheme: String,
}

#[derive(Deserialize)]
pub struct CreateCertReq {
    pub domain: String,
    pub email: String,
}

pub fn router(app_state: Arc<AppState>, db_pool: DbPool) -> Router {
    let state = ApiState { app_state, db_pool };

    let serve_dir = ServeDir::new("static")
        .not_found_service(ServeFile::new("static/index.html"));

    // 인증이 필요한 API 라우터
    let protected_api = Router::new()
        .route("/hosts", get(list_hosts).post(add_host))
        .route("/hosts/{domain}", delete(delete_host_handler))
        .route("/certs", post(request_cert));

    // 전체 라우터 조립
    Router::new()
        .route("/api/login", post(login_handler))
        .nest("/api", protected_api)
        .fallback_service(serve_dir) // axum 0.8: nest_service("/", ...) 대신 fallback_service 사용
        .layer(CorsLayer::permissive()) 
        .with_state(state)
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
    if let Ok(rows) = db::get_all_hosts(&state.db_pool).await {
        let mut hosts = HashMap::new();
        for row in rows {
            hosts.insert(row.domain, HostConfig {
                target: row.target,
                scheme: row.scheme,
            });
        }
        state.app_state.update_config(ProxyConfig { hosts });
        tracing::info!("♻️ State synced with DB");
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
    axum::extract::Path(domain): axum::extract::Path<String>,
) -> StatusCode {
    if let Err(e) = db::delete_host(&state.db_pool, &domain).await {
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

// AuthError에 InternalServerError 변형 추가 필요 (간단히 처리)
impl From<sqlx::Error> for AuthError {
    fn from(_: sqlx::Error) -> Self {
        AuthError::TokenCreation // 임시로 500 에러 매핑
    }
}