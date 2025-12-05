use axum::{
    extract::{State, Json, Path as AxumPath},
    http::StatusCode,
};
use crate::api::{ApiState, types::{UserRes, CreateUserReq, UpdateUserReq, ChangePasswordReq}, sync_state};
use crate::auth::{self, Claims};
use crate::db;

pub async fn list_users(
    claims: Claims,
    State(state): State<ApiState>,
) -> Result<Json<Vec<UserRes>>, StatusCode> {
    // Admin만 사용자 목록 조회 가능
    if !claims.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }
    
    match db::get_all_users(&state.db_pool).await {
        Ok(users) => Ok(Json(users.into_iter().map(|u| UserRes {
            id: u.id,
            username: u.username,
            role: u.role,
            created_at: u.created_at,
            last_login: u.last_login,
        }).collect())),
        Err(e) => {
            tracing::error!("Failed to fetch users: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_user_handler(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateUserReq>,
) -> StatusCode {
    // Admin만 사용자 생성 가능
    if !claims.is_admin() {
        return StatusCode::FORBIDDEN;
    }

    let role = payload.role.unwrap_or_else(|| "viewer".to_string());
    
    // 역할 검증
    if role != "admin" && role != "operator" && role != "viewer" {
        return StatusCode::BAD_REQUEST;
    }
    
    let hash = match auth::hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    
    match db::create_user_with_role(&state.db_pool, &payload.username, &hash, &role).await {
        Ok(user_id) => {
            // 감사 로그
            let _ = db::insert_audit_log(
                &state.db_pool,
                &claims.sub,
                Some(claims.user_id),
                "create",
                "user",
                Some(&user_id.to_string()),
                Some(&format!("Created user: {} with role: {}", payload.username, role)),
                None,
            ).await;
            StatusCode::CREATED
        }
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn update_user_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(user_id): AxumPath<i64>,
    Json(payload): Json<UpdateUserReq>,
) -> StatusCode {
    // Admin만 사용자 수정 가능
    if !claims.is_admin() {
        return StatusCode::FORBIDDEN;
    }
    
    // 자기 자신의 역할은 수정 불가 (실수 방지)
    if user_id == claims.user_id && payload.role.is_some() {
        return StatusCode::BAD_REQUEST;
    }
    
    if let Some(ref role) = payload.role {
        if role != "admin" && role != "operator" && role != "viewer" {
            return StatusCode::BAD_REQUEST;
        }
        if let Err(e) = db::update_user_role(&state.db_pool, user_id, role).await {
            tracing::error!("Failed to update user role: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    
    if let Some(ref password) = payload.password {
        let hash = match auth::hash_password(password) {
            Ok(h) => h,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        };
        if let Err(e) = db::update_user_password(&state.db_pool, user_id, &hash).await {
            tracing::error!("Failed to update user password: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    
    // 감사 로그
    let details = format!(
        "Updated user ID {}: role={:?}, password_changed={}",
        user_id,
        payload.role,
        payload.password.is_some()
    );
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "update",
        "user",
        Some(&user_id.to_string()),
        Some(&details),
        None,
    ).await;
    
    StatusCode::OK
}

pub async fn delete_user_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(user_id): AxumPath<i64>,
) -> StatusCode {
    // Admin만 사용자 삭제 가능
    if !claims.is_admin() {
        return StatusCode::FORBIDDEN;
    }
    
    // 자기 자신은 삭제 불가
    if user_id == claims.user_id {
        return StatusCode::BAD_REQUEST;
    }
    
    if let Err(e) = db::delete_user(&state.db_pool, user_id).await {
        tracing::error!("Failed to delete user: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "user",
        Some(&user_id.to_string()),
        Some("User deleted"),
        None,
    ).await;
    
    StatusCode::OK
}

pub async fn get_current_user(
    claims: Claims,
    State(state): State<ApiState>,
) -> Result<Json<UserRes>, StatusCode> {
    match db::get_user_full(&state.db_pool, &claims.sub).await {
        Ok(Some(user)) => Ok(Json(UserRes {
            id: user.id,
            username: user.username,
            role: user.role,
            created_at: user.created_at,
            last_login: user.last_login,
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to fetch current user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn change_own_password(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<ChangePasswordReq>,
) -> StatusCode {
    // 현재 비밀번호 확인
    let user = match db::get_user_full(&state.db_pool, &claims.sub).await {
        Ok(Some(u)) => u,
        _ => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    
    if !auth::verify_password(&payload.current_password, &user.password_hash) {
        return StatusCode::UNAUTHORIZED;
    }
    
    let new_hash = match auth::hash_password(&payload.new_password) {
        Ok(h) => h,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    
    if let Err(e) = db::update_user_password(&state.db_pool, claims.user_id, &new_hash).await {
        tracing::error!("Failed to change password: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "change_password",
        "user",
        Some(&claims.user_id.to_string()),
        Some("User changed their own password"),
        None,
    ).await;
    
    StatusCode::OK
}
