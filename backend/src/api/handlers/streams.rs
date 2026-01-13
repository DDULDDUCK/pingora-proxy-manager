use crate::api::{
    types::{CreateStreamReq, StreamRes},
    ApiState,
};
use crate::auth::Claims;
use crate::db;
use crate::error::AppError;
use axum::{
    extract::{Json, Path as AxumPath, State},
    http::StatusCode,
};

pub async fn list_streams(
    _: Claims,
    State(state): State<ApiState>,
) -> Result<Json<Vec<StreamRes>>, AppError> {
    let rows = db::get_all_streams(&state.db_pool).await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| StreamRes {
                id: r.id,
                listen_port: r.listen_port,
                forward_host: r.forward_host,
                forward_port: r.forward_port,
                protocol: r.protocol,
            })
            .collect(),
    ))
}

pub async fn add_stream(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateStreamReq>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 스트림 생성 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let protocol = payload
        .protocol
        .clone()
        .unwrap_or_else(|| "tcp".to_string());

    db::upsert_stream(
        &state.db_pool,
        payload.listen_port as i64,
        &payload.forward_host,
        payload.forward_port as i64,
        &protocol,
    )
    .await?;

    // 감사 로그
    let details = format!(
        "listen_port={}, forward={}:{}, protocol={}",
        payload.listen_port, payload.forward_host, payload.forward_port, protocol
    );
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "create",
        "stream",
        Some(&payload.listen_port.to_string()),
        Some(&details),
        None,
    )
    .await;

    state
        .stream_manager
        .start_stream(
            payload.listen_port,
            &payload.forward_host,
            payload.forward_port,
            &protocol,
        )
        .await;
    Ok(StatusCode::CREATED)
}

pub async fn delete_stream_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(port): AxumPath<u16>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 스트림 삭제 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    db::delete_stream(&state.db_pool, port as i64).await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "stream",
        Some(&port.to_string()),
        Some(&format!("Deleted stream on port {}", port)),
        None,
    )
    .await;

    state.stream_manager.stop_stream(port);
    Ok(StatusCode::OK)
}
