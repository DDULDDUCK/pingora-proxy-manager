use axum::{
    extract::{State, Json, Path as AxumPath},
    http::StatusCode,
};
use crate::api::{ApiState, types::{StreamRes, CreateStreamReq}};
use crate::auth::Claims;
use crate::db;

pub async fn list_streams(
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

pub async fn add_stream(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateStreamReq>,
) -> StatusCode {
    // Operator 이상만 스트림 생성 가능
    if !claims.can_manage_hosts() {
        return StatusCode::FORBIDDEN;
    }
    
    let protocol = payload.protocol.clone().unwrap_or_else(|| "tcp".to_string());
    if let Err(e) = db::upsert_stream(&state.db_pool, payload.listen_port as i64, &payload.forward_host, payload.forward_port as i64, &protocol).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
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
    ).await;
    
    state.stream_manager.start_stream(payload.listen_port, &payload.forward_host, payload.forward_port, &protocol).await;
    StatusCode::CREATED
}

pub async fn delete_stream_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(port): AxumPath<u16>,
) -> StatusCode {
    // Operator 이상만 스트림 삭제 가능
    if !claims.can_manage_hosts() {
        return StatusCode::FORBIDDEN;
    }
    
    if let Err(e) = db::delete_stream(&state.db_pool, port as i64).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
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
    ).await;
    
    state.stream_manager.stop_stream(port);
    StatusCode::OK
}
