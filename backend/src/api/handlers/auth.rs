use crate::api::{
    types::{LoginReq, LoginRes},
    ApiState,
};
use crate::auth;
use crate::db;
use crate::error::AppError;
use axum::{extract::State, Json};

pub async fn login_handler(
    State(state): State<ApiState>,
    Json(payload): Json<LoginReq>,
) -> Result<Json<LoginRes>, AppError> {
    let user = db::get_user_full(&state.db_pool, &payload.username)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid username or password".to_string()))?;

    if auth::verify_password(&payload.password, &user.password_hash) {
        // 마지막 로그인 시간 업데이트
        let _ = db::update_last_login(&state.db_pool, user.id).await;

        // 로그인 감사 로그
        let _ = db::insert_audit_log(
            &state.db_pool,
            &payload.username,
            Some(user.id),
            "login",
            "session",
            None,
            None,
            None,
        )
        .await;

        let token = auth::create_jwt(&payload.username, user.id, &user.role)
            .map_err(|e| AppError::Auth(format!("{:?}", e)))?;
        Ok(Json(LoginRes { token }))
    } else {
        Err(AppError::Auth("Invalid username or password".to_string()))
    }
}
