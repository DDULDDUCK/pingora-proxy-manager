use super::DbPool;
use serde::Serialize;

#[derive(sqlx::FromRow, Debug, Clone, Serialize)]
pub struct TrafficStatRow {
    pub id: i64,
    pub timestamp: i64,
    pub total_requests: i64,
    pub total_bytes: i64,
    pub status_2xx: i64,
    pub status_4xx: i64,
    pub status_5xx: i64,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize)]
pub struct AuditLogRow {
    pub id: i64,
    pub timestamp: i64,
    pub user_id: Option<i64>,
    pub username: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
}

/// Inserts a new traffic statistics record into the database.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `timestamp` - Timestamp of the statistics (Unix epoch)
/// * `reqs` - Number of requests
/// * `bytes` - Total bytes transferred
/// * `s2xx` - Number of 2xx status codes
/// * `s4xx` - Number of 4xx status codes
/// * `s5xx` - Number of 5xx status codes
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn insert_traffic_stat(
    pool: &DbPool,
    timestamp: i64,
    reqs: u64,
    bytes: u64,
    s2xx: u64,
    s4xx: u64,
    s5xx: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO traffic_stats (timestamp, total_requests, total_bytes, status_2xx, status_4xx, status_5xx)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(timestamp)
    .bind(reqs as i64)
    .bind(bytes as i64)
    .bind(s2xx as i64)
    .bind(s4xx as i64)
    .bind(s5xx as i64)
    .execute(pool)
    .await?;
    Ok(())
}

/// Retrieves traffic statistics for a given time range.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `start_ts` - Start timestamp (Unix epoch)
/// * `end_ts` - End timestamp (Unix epoch)
///
/// # Returns
/// * `Result<Vec<TrafficStatRow>, sqlx::Error>` - A list of traffic statistics within the range or a database error
pub async fn get_traffic_stats(
    pool: &DbPool,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<TrafficStatRow>, sqlx::Error> {
    sqlx::query_as::<_, TrafficStatRow>(
        "SELECT * FROM traffic_stats WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp ASC"
    )
    .bind(start_ts)
    .bind(end_ts)
    .fetch_all(pool)
    .await
}

/// Inserts a new audit log record.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `username` - Name of the user performing the action
/// * `user_id` - Optional ID of the user performing the action
/// * `action` - Action performed (e.g., 'create_host')
/// * `resource_type` - Type of resource affected (e.g., 'host')
/// * `resource_id` - Optional ID of the affected resource
/// * `details` - Optional details about the action
/// * `ip_address` - Optional IP address of the user
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn insert_audit_log(
    pool: &DbPool,
    username: &str,
    user_id: Option<i64>,
    action: &str,
    resource_type: &str,
    resource_id: Option<&str>,
    details: Option<&str>,
    ip_address: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (username, user_id, action, resource_type, resource_id, details, ip_address)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(username)
    .bind(user_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(details)
    .bind(ip_address)
    .execute(pool)
    .await?;
    Ok(())
}

/// Retrieves audit logs with pagination.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `limit` - Maximum number of records to retrieve
/// * `offset` - Number of records to skip
///
/// # Returns
/// * `Result<Vec<AuditLogRow>, sqlx::Error>` - A list of audit logs or a database error
pub async fn get_audit_logs(
    pool: &DbPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogRow>(
        "SELECT * FROM audit_logs ORDER BY timestamp DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Retrieves audit logs filtered by username.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `username` - Username to filter by
/// * `limit` - Maximum number of records to retrieve
///
/// # Returns
/// * `Result<Vec<AuditLogRow>, sqlx::Error>` - A list of audit logs for the user or a database error
pub async fn get_audit_logs_by_user(
    pool: &DbPool,
    username: &str,
    limit: i64,
) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogRow>(
        "SELECT * FROM audit_logs WHERE username = ? ORDER BY timestamp DESC LIMIT ?",
    )
    .bind(username)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Retrieves audit logs filtered by resource type.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `resource_type` - Resource type to filter by
/// * `limit` - Maximum number of records to retrieve
///
/// # Returns
/// * `Result<Vec<AuditLogRow>, sqlx::Error>` - A list of audit logs for the resource type or a database error
pub async fn get_audit_logs_by_resource(
    pool: &DbPool,
    resource_type: &str,
    limit: i64,
) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogRow>(
        "SELECT * FROM audit_logs WHERE resource_type = ? ORDER BY timestamp DESC LIMIT ?",
    )
    .bind(resource_type)
    .bind(limit)
    .fetch_all(pool)
    .await
}
