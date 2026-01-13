use crate::api::{
    sync_state,
    types::{
        CreateHeaderReq, CreateHostReq, CreateLocationReq, DeleteLocationQuery, HeaderRes, HostRes,
        LocationRes,
    },
    ApiState,
};
use crate::auth::Claims;
use crate::db;
use crate::error::AppError;
use axum::{
    extract::{Json, Path as AxumPath, Query, State},
    http::StatusCode,
};

pub async fn list_hosts(
    _: Claims,
    State(state): State<ApiState>,
) -> Result<Json<Vec<HostRes>>, AppError> {
    let hosts = state.app_state.config.load();
    let res: Vec<HostRes> = hosts
        .hosts
        .iter()
        .map(|(d, c)| HostRes {
            domain: d.clone(),
            // Join Vec<String> back to String for API compatibility
            target: c.targets.join(","),
            scheme: c.scheme.clone(),
            ssl_forced: c.ssl_forced,
            verify_ssl: c.verify_ssl,
            redirect_to: c.redirect_to.clone(),
            redirect_status: c.redirect_status,
            // Need to map locations too because they also have targets: Vec<String>
            locations: c
                .locations
                .iter()
                .map(|loc| LocationRes {
                    path: loc.path.clone(),
                    target: loc.targets.join(","),
                    scheme: loc.scheme.clone(),
                    rewrite: loc.rewrite,
                    verify_ssl: loc.verify_ssl,
                })
                .collect(),
            access_list_id: c.access_list_id,
            headers: c
                .headers
                .iter()
                .map(|h| HeaderRes {
                    id: h.id,
                    name: h.name.clone(),
                    value: h.value.clone(),
                    target: h.target.clone(),
                })
                .collect(),
        })
        .collect();
    Ok(Json(res))
}

pub async fn add_host(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateHostReq>,
) -> Result<StatusCode, AppError> {
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let scheme = payload.scheme.clone().unwrap_or_else(|| "http".to_string());
    let ssl_forced = payload.ssl_forced.unwrap_or(false);
    let verify_ssl = payload.verify_ssl.unwrap_or(true);
    let redirect_status = payload.redirect_status.unwrap_or(301);

    let is_update = db::get_host_id(&state.db_pool, &payload.domain)
        .await?
        .is_some();

    db::upsert_host(
        &state.db_pool,
        &payload.domain,
        &payload.target, // DB expects String, so this is fine (CSV)
        &scheme,
        ssl_forced,
        verify_ssl,
        payload.redirect_to.clone(),
        redirect_status,
        payload.access_list_id,
    )
    .await?;

    let action = if is_update { "update" } else { "create" };
    let details = format!(
        "domain={}, target={}, scheme={}, ssl_forced={}, verify_ssl={}, redirect_to={:?}, access_list_id={:?}",
        payload.domain,
        payload.target,
        scheme,
        ssl_forced,
        verify_ssl,
        payload.redirect_to,
        payload.access_list_id
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
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::CREATED)
}

pub async fn delete_host_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
) -> Result<StatusCode, AppError> {
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    db::delete_host(&state.db_pool, &domain).await?;

    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "host",
        Some(&domain),
        Some(&format!("Deleted host: {}", domain)),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::OK)
}

pub async fn add_location(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
    Json(payload): Json<CreateLocationReq>,
) -> Result<StatusCode, AppError> {
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let host_id = db::get_host_id(&state.db_pool, &domain)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Host {} not found", domain)))?;

    let scheme = payload.scheme.clone().unwrap_or_else(|| "http".to_string());
    let rewrite = payload.rewrite.unwrap_or(false);
    let verify_ssl = payload.verify_ssl.unwrap_or(true);

    db::upsert_location(
        &state.db_pool,
        host_id,
        &payload.path,
        &payload.target, // DB expects String (CSV)
        &scheme,
        rewrite,
        verify_ssl,
    )
    .await?;

    let details = format!(
        "host={}, path={}, target={}, scheme={}, rewrite={}, verify_ssl={}",
        domain, payload.path, payload.target, scheme, rewrite, verify_ssl
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
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::CREATED)
}

pub async fn delete_location_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
    Query(q): Query<DeleteLocationQuery>,
) -> Result<StatusCode, AppError> {
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let host_id = db::get_host_id(&state.db_pool, &domain)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Host {} not found", domain)))?;

    db::delete_location(&state.db_pool, host_id, &q.path).await?;

    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "location",
        Some(&format!("{}:{}", domain, q.path)),
        Some(&format!("Deleted location {} from host {}", q.path, domain)),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::OK)
}

pub async fn list_host_headers(
    _: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
) -> Result<Json<Vec<HeaderRes>>, AppError> {
    let hosts = state.app_state.config.load();
    let host_config = hosts
        .hosts
        .get(&domain)
        .ok_or_else(|| AppError::NotFound(format!("Host {} not found", domain)))?;

    Ok(Json(
        host_config
            .headers
            .iter()
            .map(|h| HeaderRes {
                id: h.id,
                name: h.name.clone(),
                value: h.value.clone(),
                target: h.target.clone(),
            })
            .collect(),
    ))
}

pub async fn add_header_to_host(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(domain): AxumPath<String>,
    Json(payload): Json<CreateHeaderReq>,
) -> Result<StatusCode, AppError> {
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let host_id = db::get_host_id(&state.db_pool, &domain)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Host {} not found", domain)))?;

    db::add_header(
        &state.db_pool,
        host_id,
        &payload.name,
        &payload.value,
        &payload.target,
    )
    .await?;

    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "add",
        "header",
        Some(&format!("{}:{}", domain, payload.name)),
        Some(&format!(
            "Added header '{}: {}' to host {}",
            payload.name, payload.value, domain
        )),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::CREATED)
}

pub async fn delete_host_header(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath((domain, header_id)): AxumPath<(String, i64)>,
) -> Result<StatusCode, AppError> {
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    db::delete_header(&state.db_pool, header_id).await?;

    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "header",
        Some(&format!("{}:{}", domain, header_id)),
        Some(&format!(
            "Deleted header ID {} from host {}",
            header_id, domain
        )),
        None,
    )
    .await;

    sync_state(&state).await;
    Ok(StatusCode::OK)
}
