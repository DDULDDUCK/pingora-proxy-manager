use super::DbPool;
use crate::state::HeaderConfig;

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
pub struct HeaderRow {
    pub id: i64,
    pub host_id: i64,
    pub name: String,
    pub value: String,
    pub target: String, // 'request' or 'response'
}

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

pub async fn get_headers_by_host_id(pool: &DbPool, host_id: i64) -> Result<Vec<HeaderRow>, sqlx::Error> {
    sqlx::query_as::<_, HeaderRow>("SELECT * FROM headers WHERE host_id = ?")
        .bind(host_id)
        .fetch_all(pool)
        .await
}

pub async fn get_all_headers(pool: &DbPool) -> Result<Vec<HeaderRow>, sqlx::Error> {
    sqlx::query_as::<_, HeaderRow>("SELECT * FROM headers")
        .fetch_all(pool)
        .await
}

pub async fn add_header(pool: &DbPool, host_id: i64, name: &str, value: &str, target: &str) -> Result<i64, sqlx::Error> {
    let id = sqlx::query("INSERT INTO headers (host_id, name, value, target) VALUES (?, ?, ?, ?)")
        .bind(host_id)
        .bind(name)
        .bind(value)
        .bind(target)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

pub async fn delete_header(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM headers WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn upsert_host(
    pool: &DbPool, 
    domain: &str, 
    target: &str, 
    scheme: &str, 
    ssl_forced: bool,
    redirect_to: Option<String>,
    redirect_status: i64,
    access_list_id: Option<i64>,
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
    .bind(access_list_id)
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
