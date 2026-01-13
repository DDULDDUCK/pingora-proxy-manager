use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::error::Error;

pub type DbPool = Pool<Sqlite>;

pub mod access_lists;
pub mod certs;
pub mod hosts;
pub mod stats;
pub mod streams;
pub mod users;

pub use access_lists::*;
pub use certs::*;
pub use hosts::*;
pub use stats::*;
pub use streams::*;
pub use users::*;

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
            verify_ssl BOOLEAN NOT NULL DEFAULT 1,
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
    let _ = sqlx::query("ALTER TABLE hosts ADD COLUMN access_list_id INTEGER")
        .execute(&pool)
        .await;

    // 마이그레이션: verify_ssl 컬럼 추가
    let _ = sqlx::query("ALTER TABLE hosts ADD COLUMN verify_ssl BOOLEAN NOT NULL DEFAULT 1")
        .execute(&pool)
        .await;

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
            verify_ssl BOOLEAN NOT NULL DEFAULT 1,
            FOREIGN KEY(host_id) REFERENCES hosts(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 마이그레이션: verify_ssl 컬럼 추가 for locations
    let _ = sqlx::query("ALTER TABLE locations ADD COLUMN verify_ssl BOOLEAN NOT NULL DEFAULT 1")
        .execute(&pool)
        .await;

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

    // DNS Providers (Certbot DNS Plugins)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dns_providers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            provider_type TEXT NOT NULL,
            credentials TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
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
            expires_at INTEGER NOT NULL,
            provider_id INTEGER
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 마이그레이션: provider_id 컬럼 추가
    let _ = sqlx::query("ALTER TABLE certs ADD COLUMN provider_id INTEGER")
        .execute(&pool)
        .await;

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
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'admin'")
        .execute(&pool)
        .await;
    let _ = sqlx::query(
        "ALTER TABLE users ADD COLUMN created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))",
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN last_login INTEGER")
        .execute(&pool)
        .await;

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
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs (timestamp);",
    )
    .execute(&pool)
    .await;
    let _ =
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_username ON audit_logs (username);")
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
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_traffic_stats_timestamp ON traffic_stats (timestamp);",
    )
    .execute(&pool)
    .await;

    Ok(pool)
}
