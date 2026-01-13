use crate::api::{
    types::{
        AuditLogQuery, AuditLogRes, ErrorPageReq, HistoryStatsQuery, LogsQuery, RealtimeStatsRes,
    },
    ApiState,
};
use crate::auth::Claims;
use crate::db::{self, TrafficStatRow};
use crate::error::AppError;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
};
use std::fs;
use std::path::Path;
use std::sync::atomic::Ordering;

pub async fn get_realtime_stats(
    _: Claims,
    State(state): State<ApiState>,
) -> Json<RealtimeStatsRes> {
    let m = &state.app_state.metrics;
    Json(RealtimeStatsRes {
        requests: m.total_requests.load(Ordering::Relaxed),
        bytes: m.total_bytes.load(Ordering::Relaxed),
        status_2xx: m.status_2xx.load(Ordering::Relaxed),
        status_4xx: m.status_4xx.load(Ordering::Relaxed),
        status_5xx: m.status_5xx.load(Ordering::Relaxed),
    })
}

pub async fn get_history_stats(
    _: Claims,
    State(state): State<ApiState>,
    Query(q): Query<HistoryStatsQuery>,
) -> Result<Json<Vec<TrafficStatRow>>, AppError> {
    let hours = q.hours.unwrap_or(24);
    let end_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Config(format!("System time error: {}", e)))?
        .as_secs() as i64;
    let start_ts = end_ts - (hours * 3600);
    let rows = db::get_traffic_stats(&state.db_pool, start_ts, end_ts).await?;
    Ok(Json(rows))
}

pub async fn get_logs(
    _: Claims,
    Query(q): Query<LogsQuery>,
) -> Result<Json<Vec<String>>, AppError> {
    let limit = q.lines.unwrap_or(100);
    let log_dir = Path::new("logs");
    let now = chrono::Local::now();
    let filename = format!("access.log.{}", now.format("%Y-%m-%d"));
    let path = log_dir.join(filename);
    if !path.exists() {
        return Ok(Json(vec!["No logs found for today.".to_string()]));
    }
    let content = fs::read_to_string(path)?;
    let lines: Vec<String> = content
        .lines()
        .rev()
        .take(limit)
        .map(|s| s.to_string())
        .collect();
    Ok(Json(lines))
}

pub async fn get_error_page(_: Claims) -> String {
    fs::read_to_string("data/templates/error.html").unwrap_or_default()
}

pub async fn update_error_page(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<ErrorPageReq>,
) -> Result<StatusCode, AppError> {
    // Admin만 설정 변경 가능
    if !claims.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can update error page".to_string(),
        ));
    }

    fs::write("data/templates/error.html", &payload.html)?;

    // 감사 로그
    let _ = db::insert_audit_log(
        &state.db_pool,
        &claims.sub,
        Some(claims.user_id),
        "update",
        "settings",
        Some("error_page"),
        Some("Updated error page template"),
        None,
    )
    .await;

    state.app_state.update_error_template(payload.html);
    Ok(StatusCode::OK)
}

// --- Audit Log Handlers ---

pub async fn get_audit_logs_handler(
    claims: Claims,
    State(state): State<ApiState>,
    Query(q): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLogRes>>, AppError> {
    // Admin만 전체 로그 조회 가능, 일반 사용자는 자신의 로그만
    let limit = q.limit.unwrap_or(100);
    let offset = q.offset.unwrap_or(0);

    let rows = if claims.is_admin() {
        // Admin: 필터 적용
        if let Some(ref username) = q.username {
            db::get_audit_logs_by_user(&state.db_pool, username, limit).await?
        } else if let Some(ref resource_type) = q.resource_type {
            db::get_audit_logs_by_resource(&state.db_pool, resource_type, limit).await?
        } else {
            db::get_audit_logs(&state.db_pool, limit, offset).await?
        }
    } else {
        // 일반 사용자: 자신의 로그만
        db::get_audit_logs_by_user(&state.db_pool, &claims.sub, limit).await?
    };

    Ok(Json(
        rows.into_iter()
            .map(|r| AuditLogRes {
                id: r.id,
                timestamp: r.timestamp,
                username: r.username,
                action: r.action,
                resource_type: r.resource_type,
                resource_id: r.resource_id,
                details: r.details,
                ip_address: r.ip_address,
            })
            .collect(),
    ))
}

pub async fn metrics_handler(State(state): State<ApiState>) -> String {
    state.prometheus_handle.render()
}
