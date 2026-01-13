use crate::api::{
    types::{ChangePasswordReq, CreateUserReq, UpdateUserReq, UserRes},
    ApiState,
};
use crate::auth::{self, Claims};
use crate::db;
use crate::error::AppError;
use axum::{
    extract::{Json, Path as AxumPath, State},
    http::StatusCode,
};

pub async fn list_users(
    claims: Claims,
    State(state): State<ApiState>,
) -> Result<Json<Vec<UserRes>>, AppError> {
    // Admin만 사용자 목록 조회 가능
    if !claims.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can list users".to_string(),
        ));
    }

    let users = db::get_all_users(&state.db_pool).await?;
    Ok(Json(
        users
            .into_iter()
            .map(|u| UserRes {
                id: u.id,
                username: u.username,
                role: u.role,
                created_at: u.created_at,
                last_login: u.last_login,
            })
            .collect(),
    ))
}

pub async fn create_user_handler(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateUserReq>,
) -> Result<StatusCode, AppError> {
    // Admin만 사용자 생성 가능
    if !claims.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can create users".to_string(),
        ));
    }

    let role = payload.role.unwrap_or_else(|| "viewer".to_string());

    // 역할 검증
    if role != "admin" && role != "operator" && role != "viewer" {
        return Err(AppError::BadRequest("Invalid role".to_string()));
    }

    let hash = auth::hash_password(&payload.password)
        .map_err(|e| AppError::Config(format!("Password hashing failed: {}", e)))?;

    let user_id =
        db::create_user_with_role(&state.db_pool, &payload.username, &hash, &role).await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "create",
        "user",
        Some(&user_id.to_string()),
        Some(&format!(
            "Created user: {} with role: {}",
            payload.username, role
        )),
        None,
    )
    .await;

    Ok(StatusCode::CREATED)
}

pub async fn update_user_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(user_id): AxumPath<i64>,
    Json(payload): Json<UpdateUserReq>,
) -> Result<StatusCode, AppError> {
    // Admin만 사용자 수정 가능
    if !claims.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can update users".to_string(),
        ));
    }

    // 자기 자신의 역할은 수정 불가 (실수 방지)
    if user_id == claims.user_id && payload.role.is_some() {
        return Err(AppError::BadRequest(
            "Cannot change your own role".to_string(),
        ));
    }

    if let Some(ref role) = payload.role {
        if role != "admin" && role != "operator" && role != "viewer" {
            return Err(AppError::BadRequest("Invalid role".to_string()));
        }
        db::update_user_role(&state.db_pool, user_id, role).await?;
    }

    if let Some(ref password) = payload.password {
        let hash = auth::hash_password(password)
            .map_err(|e| AppError::Config(format!("Password hashing failed: {}", e)))?;
        db::update_user_password(&state.db_pool, user_id, &hash).await?;
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
    )
    .await;

    Ok(StatusCode::OK)
}

pub async fn delete_user_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(user_id): AxumPath<i64>,
) -> Result<StatusCode, AppError> {
    // Admin만 사용자 삭제 가능
    if !claims.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can delete users".to_string(),
        ));
    }

    // 자기 자신은 삭제 불가
    if user_id == claims.user_id {
        return Err(AppError::BadRequest("Cannot delete yourself".to_string()));
    }

    db::delete_user(&state.db_pool, user_id).await?;

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
    )
    .await;

    Ok(StatusCode::OK)
}

pub async fn get_current_user(
    claims: Claims,
    State(state): State<ApiState>,
) -> Result<Json<UserRes>, AppError> {
    let user = db::get_user_full(&state.db_pool, &claims.sub).await?;
    match user {
        Some(user) => Ok(Json(UserRes {
            id: user.id,
            username: user.username,
            role: user.role,
            created_at: user.created_at,
            last_login: user.last_login,
        })),
        None => Err(AppError::NotFound(format!("User {} not found", claims.sub))),
    }
}

pub async fn change_own_password(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<ChangePasswordReq>,
) -> Result<StatusCode, AppError> {
    // 현재 비밀번호 확인
    let user = db::get_user_full(&state.db_pool, &claims.sub)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if !auth::verify_password(&payload.current_password, &user.password_hash) {
        return Err(AppError::Auth("Invalid current password".to_string()));
    }

    let new_hash = auth::hash_password(&payload.new_password)
        .map_err(|e| AppError::Config(format!("Password hashing failed: {}", e)))?;

    db::update_user_password(&state.db_pool, claims.user_id, &new_hash).await?;

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
    )
    .await;

    Ok(StatusCode::OK)
}
