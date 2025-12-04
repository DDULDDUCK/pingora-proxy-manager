use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::error::Error;
use serde::Serialize;

pub type DbPool = Pool<Sqlite>;

/// DB 초기화 및 스키마 생성
pub async fn init_db(db_url: &str) -> Result<DbPool, Box<dyn Error>> {
    // DB 커넥션 풀 생성
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    // 호스트 테이블 생성
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS hosts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL UNIQUE,
            target TEXT NOT NULL,
            scheme TEXT NOT NULL DEFAULT 'http',
            ssl_forced BOOLEAN NOT NULL DEFAULT 0,
            redirect_to TEXT,
            redirect_status INTEGER NOT NULL DEFAULT 301,
            access_list_id INTEGER,
            FOREIGN KEY(access_list_id) REFERENCES access_lists(id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 마이그레이션: access_list_id 컬럼이 없으면 추가 (기존 DB 호환성)
    // Note: SQLite에서 컬럼 존재 여부 확인 후 추가하는 로직은 복잡하므로, 
    // 단순하게 실패를 허용하는 방식으로 시도하거나(pragmatic approach), 
    // 아래 쿼리는 컬럼이 없을 때만 성공하도록 작성할 수는 없으므로 
    // 에러를 무시하는 방식으로 처리합니다.
    let _ = sqlx::query("ALTER TABLE hosts ADD COLUMN access_list_id INTEGER").execute(&pool).await;

    // Locations (경로별 라우팅) 테이블 생성
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            target TEXT NOT NULL,
            scheme TEXT NOT NULL DEFAULT 'http',
            rewrite BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY(host_id) REFERENCES hosts(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Stream (TCP/UDP) 테이블 생성
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS streams (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            listen_port INTEGER NOT NULL UNIQUE,
            forward_host TEXT NOT NULL,
            forward_port INTEGER NOT NULL,
            protocol TEXT NOT NULL DEFAULT 'tcp'
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Access Lists 테이블
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS access_lists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Access List Clients (Basic Auth)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS access_list_clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            list_id INTEGER NOT NULL,
            username TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            FOREIGN KEY(list_id) REFERENCES access_lists(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Access List IPs (Allow/Deny)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS access_list_ips (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            list_id INTEGER NOT NULL,
            ip_address TEXT NOT NULL,
            action TEXT NOT NULL CHECK(action IN ('allow', 'deny')),
            FOREIGN KEY(list_id) REFERENCES access_lists(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Headers (Custom Headers)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS headers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            target TEXT NOT NULL CHECK(target IN ('request', 'response')),
            FOREIGN KEY(host_id) REFERENCES hosts(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Custom Certs
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS custom_certs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL UNIQUE,
            cert_path TEXT NOT NULL,
            key_path TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 기존 인증서 테이블 (Let's Encrypt 용)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS certs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL UNIQUE,
            expires_at INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 사용자 테이블 생성 (로그인용) - role 추가
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'viewer',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            last_login INTEGER
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 마이그레이션: role 컬럼 추가 (기존 DB 호환성)
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'admin'").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN last_login INTEGER").execute(&pool).await;

    // 감사 로그(Audit Log) 테이블 생성
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            user_id INTEGER,
            username TEXT NOT NULL,
            action TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT,
            details TEXT,
            ip_address TEXT
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 인덱스 추가 (조회 속도 향상)
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs (timestamp);")
        .execute(&pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_username ON audit_logs (username);")
        .execute(&pool)
        .await;

    // 트래픽 통계 테이블 생성 (시계열)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS traffic_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            total_requests INTEGER NOT NULL,
            total_bytes INTEGER NOT NULL,
            status_2xx INTEGER NOT NULL,
            status_4xx INTEGER NOT NULL,
            status_5xx INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 인덱스 추가 (조회 속도 향상)
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_traffic_stats_timestamp ON traffic_stats (timestamp);")
        .execute(&pool)
        .await;

    Ok(pool)
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct HostRow {
    pub id: i64,
    pub domain: String,
    pub target: String,
    pub scheme: String,
    pub ssl_forced: bool,
    pub redirect_to: Option<String>,
    pub redirect_status: i64,
    pub access_list_id: Option<i64>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct LocationRow {
    pub id: i64,
    pub host_id: i64,
    pub path: String,
    pub target: String,
    pub scheme: String,
    pub rewrite: bool,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct StreamRow {
    pub id: i64,
    pub listen_port: i64,
    pub forward_host: String,
    pub forward_port: i64,
    pub protocol: String,
}

// --- Access List Structs ---
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct AccessListRow {
    pub id: i64,
    pub name: String,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct AccessListClientRow {
    pub id: i64,
    pub list_id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct AccessListIpRow {
    pub id: i64,
    pub list_id: i64,
    pub ip_address: String,
    pub action: String, // 'allow' or 'deny'
}

// --- Headers Structs ---
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct HeaderRow {
    pub id: i64,
    pub host_id: i64,
    pub name: String,
    pub value: String,
    pub target: String, // 'request' or 'response'
}

// --- DB Access Functions ---

pub async fn get_all_hosts(pool: &DbPool) -> Result<Vec<HostRow>, sqlx::Error> {
    sqlx::query_as::<_, HostRow>("SELECT * FROM hosts")
        .fetch_all(pool)
        .await
}

pub async fn get_host_id(pool: &DbPool, domain: &str) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64,)>("SELECT id FROM hosts WHERE domain = ?")
        .bind(domain)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn get_all_locations(pool: &DbPool) -> Result<Vec<LocationRow>, sqlx::Error> {
    sqlx::query_as::<_, LocationRow>("SELECT * FROM locations")
        .fetch_all(pool)
        .await
}

pub async fn get_all_streams(pool: &DbPool) -> Result<Vec<StreamRow>, sqlx::Error> {
    sqlx::query_as::<_, StreamRow>("SELECT * FROM streams")
        .fetch_all(pool)
        .await
}

// New Access List Functions
pub async fn get_all_access_lists(pool: &DbPool) -> Result<Vec<AccessListRow>, sqlx::Error> {
    sqlx::query_as::<_, AccessListRow>("SELECT * FROM access_lists").fetch_all(pool).await
}

pub async fn get_access_list_clients(pool: &DbPool) -> Result<Vec<AccessListClientRow>, sqlx::Error> {
    sqlx::query_as::<_, AccessListClientRow>("SELECT * FROM access_list_clients").fetch_all(pool).await
}

pub async fn get_access_list_ips(pool: &DbPool) -> Result<Vec<AccessListIpRow>, sqlx::Error> {
    sqlx::query_as::<_, AccessListIpRow>("SELECT * FROM access_list_ips").fetch_all(pool).await
}

// New Headers Functions
pub async fn get_all_headers(pool: &DbPool) -> Result<Vec<HeaderRow>, sqlx::Error> {
    sqlx::query_as::<_, HeaderRow>("SELECT * FROM headers").fetch_all(pool).await
}

// 👇 [수정됨] access_list_id 인자 추가 및 쿼리 반영
pub async fn upsert_host(
    pool: &DbPool, 
    domain: &str, 
    target: &str, 
    scheme: &str, 
    ssl_forced: bool,
    redirect_to: Option<String>,
    redirect_status: i64,
    access_list_id: Option<i64>, // Added
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO hosts (domain, target, scheme, ssl_forced, redirect_to, redirect_status, access_list_id)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(domain) DO UPDATE SET 
            target = excluded.target, 
            scheme = excluded.scheme,
            ssl_forced = excluded.ssl_forced,
            redirect_to = excluded.redirect_to,
            redirect_status = excluded.redirect_status,
            access_list_id = excluded.access_list_id
        "#,
    )
    .bind(domain)
    .bind(target)
    .bind(scheme)
    .bind(ssl_forced)
    .bind(redirect_to)
    .bind(redirect_status)
    .bind(access_list_id) // Added bind
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_host(pool: &DbPool, domain: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hosts WHERE domain = ?")
        .bind(domain)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn upsert_location(pool: &DbPool, host_id: i64, path: &str, target: &str, scheme: &str, rewrite: bool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM locations WHERE host_id = ? AND path = ?")
        .bind(host_id)
        .bind(path)
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO locations (host_id, path, target, scheme, rewrite) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(host_id)
    .bind(path)
    .bind(target)
    .bind(scheme)
    .bind(rewrite)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_location(pool: &DbPool, host_id: i64, path: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM locations WHERE host_id = ? AND path = ?")
        .bind(host_id)
        .bind(path)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn upsert_stream(
    pool: &DbPool,
    listen_port: i64,
    forward_host: &str,
    forward_port: i64,
    protocol: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO streams (listen_port, forward_host, forward_port, protocol)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(listen_port) DO UPDATE SET 
            forward_host = excluded.forward_host,
            forward_port = excluded.forward_port,
            protocol = excluded.protocol
        "#,
    )
    .bind(listen_port)
    .bind(forward_host)
    .bind(forward_port)
    .bind(protocol)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_stream(pool: &DbPool, listen_port: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM streams WHERE listen_port = ?")
        .bind(listen_port)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Certs ---

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CertRow {
    pub id: i64,
    pub domain: String,
    pub expires_at: i64,
}

pub async fn upsert_cert(pool: &DbPool, domain: &str, expires_at: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO certs (domain, expires_at)
        VALUES (?, ?)
        ON CONFLICT(domain) DO UPDATE SET expires_at = excluded.expires_at
        "#,
    )
    .bind(domain)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_cert(pool: &DbPool, domain: &str) -> Result<Option<CertRow>, sqlx::Error> {
    sqlx::query_as::<_, CertRow>("SELECT * FROM certs WHERE domain = ?")
        .bind(domain)
        .fetch_optional(pool)
        .await
}

pub async fn get_expiring_certs(pool: &DbPool, threshold: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CertRow>("SELECT * FROM certs WHERE expires_at < ?")
        .bind(threshold)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| r.domain).collect())
}

// --- Users ---

#[derive(sqlx::FromRow, Debug, Clone, Serialize)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: i64,
    pub last_login: Option<i64>,
}

pub async fn get_user(pool: &DbPool, username: &str) -> Result<Option<(i64, String)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>("SELECT id, password_hash FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_full(pool: &DbPool, username: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>("SELECT id, username, password_hash, role, created_at, last_login FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_all_users(pool: &DbPool) -> Result<Vec<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>("SELECT id, username, password_hash, role, created_at, last_login FROM users ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn create_user(pool: &DbPool, username: &str, password_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO users (username, password_hash, role) VALUES (?, ?, 'admin')")
        .bind(username)
        .bind(password_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_user_with_role(pool: &DbPool, username: &str, password_hash: &str, role: &str) -> Result<i64, sqlx::Error> {
    let id = sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)")
        .bind(username)
        .bind(password_hash)
        .bind(role)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

pub async fn update_user_password(pool: &DbPool, user_id: i64, password_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(password_hash)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_user_role(pool: &DbPool, user_id: i64, role: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(role)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_user(pool: &DbPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_last_login(pool: &DbPool, user_id: i64) -> Result<(), sqlx::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Audit Logs ---

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

pub async fn get_audit_logs(pool: &DbPool, limit: i64, offset: i64) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogRow>(
        "SELECT * FROM audit_logs ORDER BY timestamp DESC LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_audit_logs_by_user(pool: &DbPool, username: &str, limit: i64) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogRow>(
        "SELECT * FROM audit_logs WHERE username = ? ORDER BY timestamp DESC LIMIT ?"
    )
    .bind(username)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_audit_logs_by_resource(pool: &DbPool, resource_type: &str, limit: i64) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogRow>(
        "SELECT * FROM audit_logs WHERE resource_type = ? ORDER BY timestamp DESC LIMIT ?"
    )
    .bind(resource_type)
    .bind(limit)
    .fetch_all(pool)
    .await
}

// --- Stats ---

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

pub async fn get_traffic_stats(pool: &DbPool, start_ts: i64, end_ts: i64) -> Result<Vec<TrafficStatRow>, sqlx::Error> {
    sqlx::query_as::<_, TrafficStatRow>(
        "SELECT * FROM traffic_stats WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp ASC"
    )
    .bind(start_ts)
    .bind(end_ts)
    .fetch_all(pool)
    .await
}

// --- Access List DB Helpers ---

// 👇 [수정됨] 이름 변경: insert_access_list -> create_access_list
pub async fn create_access_list(pool: &DbPool, name: &str) -> Result<i64, sqlx::Error> {
    let id = sqlx::query("INSERT INTO access_lists (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

pub async fn delete_access_list(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM access_lists WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_access_list_client(pool: &DbPool, list_id: i64, username: &str, password_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO access_list_clients (list_id, username, password_hash) VALUES (?, ?, ?)")
        .bind(list_id)
        .bind(username)
        .bind(password_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_access_list_client(pool: &DbPool, list_id: i64, username: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM access_list_clients WHERE list_id = ? AND username = ?")
        .bind(list_id)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_access_list_ip(pool: &DbPool, list_id: i64, ip: &str, action: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO access_list_ips (list_id, ip_address, action) VALUES (?, ?, ?)")
        .bind(list_id)
        .bind(ip)
        .bind(action)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_access_list_ip(pool: &DbPool, list_id: i64, ip: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM access_list_ips WHERE list_id = ? AND ip_address = ?")
        .bind(list_id)
        .bind(ip)
        .execute(pool)
        .await?;
    Ok(())
}