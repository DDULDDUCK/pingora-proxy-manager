use axum::{
    extract::{State, Json, Query},
    http::StatusCode,
};
use crate::api::{ApiState, types::{RealtimeStatsRes, HistoryStatsQuery, LogsQuery, ErrorPageReq, AuditLogQuery, AuditLogRes}};
use crate::auth::Claims;
use crate::db::{self, TrafficStatRow};
use std::path::Path;
use std::fs;
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
) -> Json<Vec<TrafficStatRow>> {
    let hours = q.hours.unwrap_or(24);
    let end_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let start_ts = end_ts - (hours * 3600);
    match db::get_traffic_stats(&state.db_pool, start_ts, end_ts).await {
        Ok(rows) => Json(rows),
        Err(e) => {
            tracing::error!("DB Stats Error: {}", e);
            Json(vec![])
        }
    }
}

pub async fn get_logs(
    _: Claims,
    Query(q): Query<LogsQuery>,
) -> Json<Vec<String>> {
    let limit = q.lines.unwrap_or(100);
    let log_dir = Path::new("logs");
    let now = chrono::Local::now();
    let filename = format!("access.log.{}", now.format("%Y-%m-%d"));
    let path = log_dir.join(filename);
    if !path.exists() { return Json(vec!["No logs found for today.".to_string()]); }
    match fs::read_to_string(path) {
        Ok(content) => {
            let lines: Vec<String> = content.lines().rev().take(limit).map(|s| s.to_string()).collect();
            Json(lines)
        }
        Err(e) => Json(vec![format!("Failed to read log file: {}", e)]),
    }
}

pub async fn get_error_page(_: Claims) -> String {
    fs::read_to_string("data/templates/error.html").unwrap_or_default()
}

pub async fn update_error_page(
    claims: Claims,
    State(state): State<ApiState>,
    Json(payload): Json<ErrorPageReq>,
) -> StatusCode {
    // Admin만 설정 변경 가능
    if !claims.is_admin() {
        return StatusCode::FORBIDDEN;
    }
    
    if let Err(e) = fs::write("data/templates/error.html", &payload.html) {
        tracing::error!("Failed to write error template: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    
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
    ).await;
    
    state.app_state.update_error_template(payload.html);
    StatusCode::OK
}

// --- Audit Log Handlers ---

pub async fn get_audit_logs_handler(
    claims: Claims,
    State(state): State<ApiState>,
    Query(q): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLogRes>>, StatusCode> {
    // Admin만 전체 로그 조회 가능, 일반 사용자는 자신의 로그만
    let limit = q.limit.unwrap_or(100);
    let offset = q.offset.unwrap_or(0);
    
    let logs = if claims.is_admin() {
        // Admin: 필터 적용
        if let Some(ref username) = q.username {
            db::get_audit_logs_by_user(&state.db_pool, username, limit).await
        } else if let Some(ref resource_type) = q.resource_type {
            db::get_audit_logs_by_resource(&state.db_pool, resource_type, limit).await
        } else {
            db::get_audit_logs(&state.db_pool, limit, offset).await
        }
    } else {
        // 일반 사용자: 자신의 로그만
        db::get_audit_logs_by_user(&state.db_pool, &claims.sub, limit).await
    };
    
    match logs {
        Ok(rows) => Ok(Json(rows.into_iter().map(|r| AuditLogRes {
            id: r.id,
            timestamp: r.timestamp,
            username: r.username,
            action: r.action,
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            details: r.details,
            ip_address: r.ip_address,
        }).collect())),
        Err(e) => {
            tracing::error!("Failed to fetch audit logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn metrics_handler(State(state): State<ApiState>) -> String {
    state.prometheus_handle.render()
}
