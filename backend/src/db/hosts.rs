use super::DbPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct HostRow {
    pub id: i64,
    pub domain: String,
    pub target: String,
    pub scheme: String,
    pub ssl_forced: bool,
    pub verify_ssl: bool,
    pub connection_timeout_ms: Option<i64>,
    pub read_timeout_ms: Option<i64>,
    pub write_timeout_ms: Option<i64>,
    pub max_request_body_bytes: Option<i64>,
    pub redirect_to: Option<String>,
    pub redirect_status: i64,
    pub access_list_id: Option<i64>,
    pub upstream_sni: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct LocationRow {
    pub id: i64,
    pub host_id: i64,
    pub path: String,
    pub target: String,
    pub scheme: String,
    pub rewrite: bool,
    pub verify_ssl: bool,
    pub upstream_sni: Option<String>,
    pub connection_timeout_ms: Option<i64>,
    pub read_timeout_ms: Option<i64>,
    pub write_timeout_ms: Option<i64>,
    pub max_request_body_bytes: Option<i64>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct HeaderRow {
    pub id: i64,
    pub host_id: i64,
    pub name: String,
    pub value: String,
    pub target: String, // 'request' or 'response'
}

pub struct UpsertHostParams<'a> {
    pub domain: &'a str,
    pub target: &'a str,
    pub scheme: &'a str,
    pub ssl_forced: bool,
    pub verify_ssl: bool,
    pub upstream_sni: Option<&'a str>,
    pub connection_timeout_ms: Option<i64>,
    pub read_timeout_ms: Option<i64>,
    pub write_timeout_ms: Option<i64>,
    pub max_request_body_bytes: Option<i64>,
    pub redirect_to: Option<&'a str>,
    pub redirect_status: i64,
    pub access_list_id: Option<i64>,
}

pub struct UpsertLocationParams<'a> {
    pub host_id: i64,
    pub path: &'a str,
    pub target: &'a str,
    pub scheme: &'a str,
    pub rewrite: bool,
    pub verify_ssl: bool,
    pub upstream_sni: Option<&'a str>,
    pub connection_timeout_ms: Option<i64>,
    pub read_timeout_ms: Option<i64>,
    pub write_timeout_ms: Option<i64>,
    pub max_request_body_bytes: Option<i64>,
}

/// Retrieves all configured hosts from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<HostRow>, sqlx::Error>` - A list of all hosts or a database error
pub async fn get_all_hosts(pool: &DbPool) -> Result<Vec<HostRow>, sqlx::Error> {
    sqlx::query_as::<_, HostRow>("SELECT * FROM hosts")
        .fetch_all(pool)
        .await
}

/// Retrieves the ID of a host by its domain name.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `domain` - Domain name to search for
///
/// # Returns
/// * `Result<Option<i64>, sqlx::Error>` - The host ID if found, None if not found, or a database error
pub async fn get_host_id(pool: &DbPool, domain: &str) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64,)>("SELECT id FROM hosts WHERE domain = ?")
        .bind(domain)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

/// Retrieves all configured locations from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<LocationRow>, sqlx::Error>` - A list of all locations or a database error
pub async fn get_all_locations(pool: &DbPool) -> Result<Vec<LocationRow>, sqlx::Error> {
    sqlx::query_as::<_, LocationRow>("SELECT * FROM locations")
        .fetch_all(pool)
        .await
}

/// Retrieves all custom headers for a specific host.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `host_id` - ID of the host
///
/// # Returns
/// * `Result<Vec<HeaderRow>, sqlx::Error>` - A list of headers for the host or a database error
pub async fn get_headers_by_host_id(
    pool: &DbPool,
    host_id: i64,
) -> Result<Vec<HeaderRow>, sqlx::Error> {
    sqlx::query_as::<_, HeaderRow>("SELECT * FROM headers WHERE host_id = ?")
        .bind(host_id)
        .fetch_all(pool)
        .await
}

/// Retrieves all custom headers from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<HeaderRow>, sqlx::Error>` - A list of all headers or a database error
pub async fn get_all_headers(pool: &DbPool) -> Result<Vec<HeaderRow>, sqlx::Error> {
    sqlx::query_as::<_, HeaderRow>("SELECT * FROM headers")
        .fetch_all(pool)
        .await
}

/// Adds a new custom header to a host.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `host_id` - ID of the host to attach the header to
/// * `name` - Header name
/// * `value` - Header value
/// * `target` - Target of the header ('request' or 'response')
///
/// # Returns
/// * `Result<i64, sqlx::Error>` - The ID of the newly created header or a database error
pub async fn add_header(
    pool: &DbPool,
    host_id: i64,
    name: &str,
    value: &str,
    target: &str,
) -> Result<i64, sqlx::Error> {
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

/// Deletes a custom header from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `id` - ID of the header to delete
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn delete_header(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM headers WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Inserts or updates a host configuration.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `domain` - Domain name
/// * `target` - Upstream target URL
/// * `scheme` - Upstream scheme ('http' or 'https')
/// * `ssl_forced` - Whether to force HTTPS
/// * `verify_ssl` - Whether to verify upstream SSL certificates
/// * `redirect_to` - Optional redirect URL
/// * `redirect_status` - HTTP status code for redirect
/// * `access_list_id` - Optional ID of the access list to apply
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn upsert_host(pool: &DbPool, params: UpsertHostParams<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO hosts (domain, target, scheme, ssl_forced, verify_ssl, upstream_sni, connection_timeout_ms, read_timeout_ms, write_timeout_ms, max_request_body_bytes, redirect_to, redirect_status, access_list_id)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(domain) DO UPDATE SET 
            target = excluded.target, 
            scheme = excluded.scheme,
            ssl_forced = excluded.ssl_forced,
            verify_ssl = excluded.verify_ssl,
            upstream_sni = excluded.upstream_sni,
            connection_timeout_ms = excluded.connection_timeout_ms,
            read_timeout_ms = excluded.read_timeout_ms,
            write_timeout_ms = excluded.write_timeout_ms,
            max_request_body_bytes = excluded.max_request_body_bytes,
            redirect_to = excluded.redirect_to,
            redirect_status = excluded.redirect_status,
            access_list_id = excluded.access_list_id
        "#,
    )
    .bind(params.domain)
    .bind(params.target)
    .bind(params.scheme)
    .bind(params.ssl_forced)
    .bind(params.verify_ssl)
    .bind(params.upstream_sni)
    .bind(params.connection_timeout_ms)
    .bind(params.read_timeout_ms)
    .bind(params.write_timeout_ms)
    .bind(params.max_request_body_bytes)
    .bind(params.redirect_to)
    .bind(params.redirect_status)
    .bind(params.access_list_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Deletes a host by its domain name.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `domain` - Domain name to delete
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn delete_host(pool: &DbPool, domain: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hosts WHERE domain = ?")
        .bind(domain)
        .execute(pool)
        .await?;
    Ok(())
}

/// Inserts or updates a location configuration for a host.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `host_id` - ID of the host
/// * `path` - Location path
/// * `target` - Upstream target URL
/// * `scheme` - Upstream scheme
/// * `rewrite` - Whether to enable path rewriting
/// * `verify_ssl` - Whether to verify upstream SSL certificates
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn upsert_location(
    pool: &DbPool,
    params: UpsertLocationParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM locations WHERE host_id = ? AND path = ?")
        .bind(params.host_id)
        .bind(params.path)
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO locations (host_id, path, target, scheme, rewrite, verify_ssl, upstream_sni, connection_timeout_ms, read_timeout_ms, write_timeout_ms, max_request_body_bytes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(params.host_id)
    .bind(params.path)
    .bind(params.target)
    .bind(params.scheme)
    .bind(params.rewrite)
    .bind(params.verify_ssl)
    .bind(params.upstream_sni)
    .bind(params.connection_timeout_ms)
    .bind(params.read_timeout_ms)
    .bind(params.write_timeout_ms)
    .bind(params.max_request_body_bytes)
    .execute(pool)
    .await?;
    Ok(())
}

/// Deletes a location by host ID and path.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `host_id` - ID of the host
/// * `path` - Path of the location to delete
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn delete_location(pool: &DbPool, host_id: i64, path: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM locations WHERE host_id = ? AND path = ?")
        .bind(host_id)
        .bind(path)
        .execute(pool)
        .await?;
    Ok(())
}
