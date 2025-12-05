use axum::{
    extract::{State, Json, Query, Path as AxumPath},
    http::StatusCode,
};
use crate::api::{ApiState, types::{CreateHostReq, HostRes, CreateLocationReq, DeleteLocationQuery}, sync_state};
use crate::auth::Claims;
use crate::db;

pub async fn list_hosts(
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

pub async fn add_host(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateHostReq>,
) -> StatusCode {
    // Operator 이상만 호스트 생성/수정 가능
    if !claims.can_manage_hosts() {
        return StatusCode::FORBIDDEN;
    }
    
    let scheme = payload.scheme.clone().unwrap_or_else(|| "http".to_string());
    let ssl_forced = payload.ssl_forced.unwrap_or(false);
    let redirect_status = payload.redirect_status.unwrap_or(301);
    
    // 기존 호스트가 있는지 확인 (create vs update 구분)
    let is_update = db::get_host_id(&state.db_pool, &payload.domain).await.ok().flatten().is_some();
    
    match db::upsert_host(
        &state.db_pool,
        &payload.domain,
        &payload.target,
        &scheme,
        ssl_forced,
        payload.redirect_to.clone(),
        redirect_status,
        payload.access_list_id,
    ).await {
        Ok(_) => {
            // 감사 로그
            let action = if is_update { "update" } else { "create" };
            let details = format!(
                "domain={}, target={}, scheme={}, ssl_forced={}, redirect_to={:?}, access_list_id={:?}",
                payload.domain, payload.target, scheme, ssl_forced, payload.redirect_to, payload.access_list_id
            );
            let _ = db::insert_audit_log(
                &state.db_pool,
                &claims.sub,
                Some(claims.user_id),
                action,
                "host",
                Some(&payload.domain),
                Some(&details),
                None,
            ).await;
            
            sync_state(&state).await;
            StatusCode::CREATED
        }
        Err(e) => {
            tracing::error!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn delete_host_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
) -> StatusCode {
    // Operator 이상만 호스트 삭제 가능
    if !claims.can_manage_hosts() {
        return StatusCode::FORBIDDEN;
    }
    
    if let Err(e) = db::delete_host(&state.db_pool, &domain).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "host",
        Some(&domain),
        Some(&format!("Deleted host: {}", domain)),
        None,
    ).await;
    
    sync_state(&state).await;
    StatusCode::OK
}

pub async fn add_location(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
    Json(payload): Json<CreateLocationReq>,
) -> StatusCode {
    // Operator 이상만 로케이션 추가 가능
    if !claims.can_manage_hosts() {
        return StatusCode::FORBIDDEN;
    }
    
    let host_id = match db::get_host_id(&state.db_pool, &domain).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let scheme = payload.scheme.clone().unwrap_or_else(|| "http".to_string());
    let rewrite = payload.rewrite.unwrap_or(false);

    if let Err(e) = db::upsert_location(&state.db_pool, host_id, &payload.path, &payload.target, &scheme, rewrite).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
    // 감사 로그
    let details = format!(
        "host={}, path={}, target={}, scheme={}, rewrite={}",
        domain, payload.path, payload.target, scheme, rewrite
    );
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "create",
        "location",
        Some(&format!("{}:{}", domain, payload.path)),
        Some(&details),
        None,
    ).await;
    
    sync_state(&state).await;
    StatusCode::CREATED
}

pub async fn delete_location_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
    Query(q): Query<DeleteLocationQuery>,
) -> StatusCode {
    // Operator 이상만 로케이션 삭제 가능
    if !claims.can_manage_hosts() {
        return StatusCode::FORBIDDEN;
    }
    
    let host_id = match db::get_host_id(&state.db_pool, &domain).await {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if let Err(e) = db::delete_location(&state.db_pool, host_id, &q.path).await {
        tracing::error!("DB Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "location",
        Some(&format!("{}:{}", domain, q.path)),
        Some(&format!("Deleted location {} from host {}", q.path, domain)),
        None,
    ).await;
    
    sync_state(&state).await;
    StatusCode::OK
}
