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
    // scheme 컬럼 추가 (http/https)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS hosts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL UNIQUE,
            target TEXT NOT NULL,
            scheme TEXT NOT NULL DEFAULT 'http'
        );
        "#,
    )
    .execute(&pool)
    .await?;

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

    // 마이그레이션: rewrite 컬럼이 없을 경우 추가 (기존 DB 호환성)
    let _ = sqlx::query("ALTER TABLE locations ADD COLUMN rewrite BOOLEAN NOT NULL DEFAULT 0")
        .execute(&pool)
        .await;

    // 인증서 테이블 생성
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

    // 사용자 테이블 생성 (로그인용)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

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
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_traffic_stats_timestamp ON traffic_stats (timestamp);")
        .execute(&pool)
        .await?;

    Ok(pool)
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct HostRow {
    pub id: i64,
    pub domain: String,
    pub target: String,
    pub scheme: String,
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

/// 모든 호스트 목록 조회
pub async fn get_all_hosts(pool: &DbPool) -> Result<Vec<HostRow>, sqlx::Error> {
    sqlx::query_as::<_, HostRow>("SELECT * FROM hosts")
        .fetch_all(pool)
        .await
}

/// 호스트 ID 조회 (도메인으로)
pub async fn get_host_id(pool: &DbPool, domain: &str) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64,)>("SELECT id FROM hosts WHERE domain = ?")
        .bind(domain)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

/// 모든 로케이션 목록 조회
pub async fn get_all_locations(pool: &DbPool) -> Result<Vec<LocationRow>, sqlx::Error> {
    sqlx::query_as::<_, LocationRow>("SELECT * FROM locations")
        .fetch_all(pool)
        .await
}

/// 호스트 추가 (이미 존재하면 업데이트)
pub async fn upsert_host(pool: &DbPool, domain: &str, target: &str, scheme: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO hosts (domain, target, scheme)
        VALUES (?, ?, ?)
        ON CONFLICT(domain) DO UPDATE SET target = excluded.target, scheme = excluded.scheme
        "#,
    )
    .bind(domain)
    .bind(target)
    .bind(scheme)
    .execute(pool)
    .await?;
    Ok(())
}

/// 호스트 삭제
pub async fn delete_host(pool: &DbPool, domain: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hosts WHERE domain = ?")
        .bind(domain)
        .execute(pool)
        .await?;
    Ok(())
}

/// 로케이션 추가 (기존 경로 있으면 덮어쓰기 - DELETE 후 INSERT)
pub async fn upsert_location(pool: &DbPool, host_id: i64, path: &str, target: &str, scheme: &str, rewrite: bool) -> Result<(), sqlx::Error> {
    // 기존 동일 경로 제거
    sqlx::query("DELETE FROM locations WHERE host_id = ? AND path = ?")
        .bind(host_id)
        .bind(path)
        .execute(pool)
        .await?;

    // 새 경로 추가
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

/// 로케이션 삭제
pub async fn delete_location(pool: &DbPool, host_id: i64, path: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM locations WHERE host_id = ? AND path = ?")
        .bind(host_id)
        .bind(path)
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

/// 만료 시간이 주어진 시간(timestamp)보다 작은 인증서 조회
pub async fn get_expiring_certs(pool: &DbPool, threshold: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CertRow>("SELECT * FROM certs WHERE expires_at < ?")
        .bind(threshold)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| r.domain).collect())
}

// --- Users ---

pub async fn get_user(pool: &DbPool, username: &str) -> Result<Option<(i64, String)>, sqlx::Error> {
    // (id, password_hash) 반환
    sqlx::query_as::<_, (i64, String)>("SELECT id, password_hash FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn create_user(pool: &DbPool, username: &str, password_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO users (username, password_hash) VALUES (?, ?)")
        .bind(username)
        .bind(password_hash)
        .execute(pool)
        .await?;
    Ok(())
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

/// 통계 저장
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

/// 통계 조회 (최근 N분/시간)
pub async fn get_traffic_stats(pool: &DbPool, start_ts: i64, end_ts: i64) -> Result<Vec<TrafficStatRow>, sqlx::Error> {
    sqlx::query_as::<_, TrafficStatRow>(
        "SELECT * FROM traffic_stats WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp ASC"
    )
    .bind(start_ts)
    .bind(end_ts)
    .fetch_all(pool)
    .await
}
