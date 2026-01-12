use crate::acme::AcmeManager;
use crate::api::{
    types::{CertRes, CreateCertReq, CreateDnsProviderReq, DnsProviderRes},
    ApiState,
};
use crate::auth::Claims;
use crate::db;
use crate::error::AppError;
use axum::{
    extract::{Json, Multipart, Path as AxumPath, State},
    http::StatusCode,
};
use std::fs;
use std::path::Path;

pub async fn request_cert(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateCertReq>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 인증서 요청 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "request",
        "certificate",
        Some(&payload.domain),
        Some(&format!(
            "Requested Let's Encrypt certificate for {} (email: {}, provider: {:?})",
            payload.domain, payload.email, payload.provider_id
        )),
        None,
    )
    .await;

    let manager = AcmeManager::new(
        state.app_state.clone(),
        state.db_pool.clone(),
        payload.email,
    );
    tokio::spawn(async move {
        if let Err(e) = manager
            .request_certificate(&payload.domain, payload.provider_id)
            .await
        {
            tracing::error!("❌ Failed to issue cert for {}: {}", payload.domain, e);
        }
    });
    Ok(StatusCode::ACCEPTED)
}

pub async fn list_certs(_: Claims, State(state): State<ApiState>) -> Result<Json<Vec<CertRes>>, AppError> {
    let rows = db::get_all_certs(&state.db_pool).await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| CertRes {
                id: r.id,
                domain: r.domain,
                expires_at: r.expires_at,
            })
            .collect(),
    ))
}

pub async fn upload_cert(
    claims: Claims,
    State(state): State<ApiState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 인증서 업로드 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let mut cert_data = None;
    let mut key_data = None;
    let mut domain = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "domain" {
            if let Ok(txt) = field.text().await {
                domain = Some(txt);
            }
        } else if name == "cert" {
            if let Ok(bytes) = field.bytes().await {
                cert_data = Some(bytes);
            }
        } else if name == "key" {
            if let Ok(bytes) = field.bytes().await {
                key_data = Some(bytes);
            }
        }
    }

    if let (Some(d), Some(c), Some(k)) = (domain, cert_data, key_data) {
        let cert_dir = Path::new("data/certs/custom");
        if !cert_dir.exists() {
            fs::create_dir_all(cert_dir)?;
        }
        let cert_path = cert_dir.join(format!("{}.crt", d));
        let key_path = cert_dir.join(format!("{}.key", d));
        
        fs::write(&cert_path, c)?;
        fs::write(&key_path, k)?;

        // 감사 로그
        let _ = db::insert_audit_log(
            &state.db_pool,
            &claims.sub,
            Some(claims.user_id),
            "upload",
            "certificate",
            Some(&d),
            Some(&format!("Uploaded custom certificate for {}", d)),
            None,
        )
        .await;

        tracing::info!("💾 Custom certificate uploaded for {}", d);
        return Ok(StatusCode::CREATED);
    }
    Err(AppError::BadRequest("Missing domain, cert, or key".to_string()))
}

pub async fn list_dns_providers(
    _: Claims,
    State(state): State<ApiState>,
) -> Result<Json<Vec<DnsProviderRes>>, AppError> {
    let rows = db::get_all_dns_providers(&state.db_pool).await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| DnsProviderRes {
                id: r.id,
                name: r.name,
                provider_type: r.provider_type,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

pub async fn create_dns_provider_handler(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<CreateDnsProviderReq>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 DNS Provider 생성 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let id = db::create_dns_provider(
        &state.db_pool,
        &payload.name,
        &payload.provider_type,
        &payload.credentials,
    )
    .await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "create",
        "dns_provider",
        Some(&id.to_string()),
        Some(&format!(
            "Created DNS provider: {} ({})",
            payload.name, payload.provider_type
        )),
        None,
    )
    .await;
    Ok(StatusCode::CREATED)
}

pub async fn delete_dns_provider_handler(
    claims: Claims,
    State(state): State<ApiState>,
    AxumPath(id): AxumPath<i64>,
) -> Result<StatusCode, AppError> {
    // Operator 이상만 DNS Provider 삭제 가능
    if !claims.can_manage_hosts() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    db::delete_dns_provider(&state.db_pool, id).await?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "delete",
        "dns_provider",
        Some(&id.to_string()),
        Some("Deleted DNS provider"),
        None,
    )
    .await;

    Ok(StatusCode::OK)
}
