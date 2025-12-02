use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::error::Error;

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

    Ok(pool)
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct HostRow {
    pub id: i64,
    pub domain: String,
    pub target: String,
    pub scheme: String,
}

/// 모든 호스트 목록 조회
pub async fn get_all_hosts(pool: &DbPool) -> Result<Vec<HostRow>, sqlx::Error> {
    sqlx::query_as::<_, HostRow>("SELECT * FROM hosts")
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
