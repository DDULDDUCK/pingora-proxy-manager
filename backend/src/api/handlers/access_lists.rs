use crate::api::{
    sync_state,
    types::{
        AccessListClientReq, AccessListClientRes, AccessListIpReq, AccessListIpRes, AccessListRes,
        CreateAccessListReq,
    },
    ApiState,
};
use crate::auth::{self, Claims};
use crate::db;
use crate::error::AppError;
use axum::{
    extract::{Json, Path as AxumPath, State},
    http::StatusCode,
};

pub async fn list_access_lists(
    _: Claims,
    State(state): State<ApiState>,
) -> Result<Json<Vec<AccessListRes>>, AppError> {
    let al_rows = db::get_all_access_lists(&state.db_pool).await?;

    let client_rows = db::get_access_list_clients(&state.db_pool)
        .await
        .unwrap_or_default();
    let ip_rows = db::get_access_list_ips(&state.db_pool)
        .await
        .unwrap_or_default();
    let mut res = Vec::new();

    for al in al_rows {
        let clients = client_rows
            .iter()
            .filter(|c| c.list_id == al.id)
            .map(|c| AccessListClientRes {
                username: c.username.clone(),
            })
            .collect();

        let ips = ip_rows
            .iter()
            .filter(|i| i.list_id == al.id)
            .map(|i| AccessListIpRes {
                ip: i.ip_address.clone(),
                action: i.action.clone(),
            })
            .collect();

        res.push(AccessListRes {
            id: al.id,
            name: al.name,
            clients,
            ips,
        });
    }
    Ok(Json(res))
}

pub async fn create_access_list(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateAccessListReq>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 액세스 리스트 생성 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let list_id = db::create_access_list(&state.db_pool, &payload.name).await?;

    let client_count = payload.clients.len();
    let ip_count = payload.ips.len();

    for client in payload.clients {
        if let Ok(hash) = auth::hash_password(&client.password) {
            let _ =
                db::add_access_list_client(&state.db_pool, list_id, &client.username, &hash).await;
        }
    }
    for ip in payload.ips {
        let _ = db::add_access_list_ip(&state.db_pool, list_id, &ip.ip, &ip.action).await;
    }

    // 감사 로그
    let details = format!(
        "name={}, clients={}, ips={}",
        payload.name, client_count, ip_count
    );
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "create",
        "access_list",
        Some(&list_id.to_string()),
        Some(&details),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::CREATED)
}

pub async fn delete_access_list_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(id): AxumPath<i64>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 액세스 리스트 삭제 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    db::delete_access_list(&state.db_pool, id).await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "access_list",
        Some(&id.to_string()),
        Some(&format!("Deleted access list ID {}", id)),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::OK)
}

pub async fn add_access_list_client_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(id): AxumPath<i64>,
    Json(payload): Json<AccessListClientReq>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 클라이언트 추가 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let hash = auth::hash_password(&payload.password)
        .map_err(|e| AppError::Config(format!("Password hashing failed: {}", e)))?;

    db::add_access_list_client(&state.db_pool, id, &payload.username, &hash).await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "add_client",
        "access_list",
        Some(&id.to_string()),
        Some(&format!(
            "Added client '{}' to access list {}",
            payload.username, id
        )),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::CREATED)
}

pub async fn delete_access_list_client_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath((id, username)): AxumPath<(i64, String)>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 클라이언트 삭제 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    db::remove_access_list_client(&state.db_pool, id, &username).await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "remove_client",
        "access_list",
        Some(&id.to_string()),
        Some(&format!(
            "Removed client '{}' from access list {}",
            username, id
        )),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::OK)
}

pub async fn add_access_list_ip_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(id): AxumPath<i64>,
    Json(payload): Json<AccessListIpReq>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 IP 추가 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    db::add_access_list_ip(&state.db_pool, id, &payload.ip, &payload.action).await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "add_ip",
        "access_list",
        Some(&id.to_string()),
        Some(&format!(
            "Added IP '{}' ({}) to access list {}",
            payload.ip, payload.action, id
        )),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::CREATED)
}

pub async fn delete_access_list_ip_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath((id, ip)): AxumPath<(i64, String)>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 IP 삭제 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    db::remove_access_list_ip(&state.db_pool, id, &ip).await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "remove_ip",
        "access_list",
        Some(&id.to_string()),
        Some(&format!("Removed IP '{}' from access list {}", ip, id)),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::OK)
}
