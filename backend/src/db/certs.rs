use super::DbPool;
use serde::Serialize;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CertRow {
    pub id: i64,
    pub domain: String,
    pub expires_at: i64,
    pub provider_id: Option<i64>,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize)]
pub struct DnsProviderRow {
    pub id: i64,
    pub name: String,
    pub provider_type: String, // 'cloudflare', 'route53', etc.
    pub credentials: String,   // JSON string
    pub created_at: i64,
}

/// Inserts or updates a certificate record.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `domain` - Domain name for the certificate
/// * `expires_at` - Expiration timestamp (Unix epoch)
/// * `provider_id` - Optional ID of the DNS provider used for renewal
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn upsert_cert(
    pool: &DbPool,
    domain: &str,
    expires_at: i64,
    provider_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO certs (domain, expires_at, provider_id)
        VALUES (?, ?, ?)
        ON CONFLICT(domain) DO UPDATE SET 
            expires_at = excluded.expires_at,
            provider_id = excluded.provider_id
        "#,
    )
    .bind(domain)
    .bind(expires_at)
    .bind(provider_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Retrieves a certificate record by domain name.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `domain` - Domain name
///
/// # Returns
/// * `Result<Option<CertRow>, sqlx::Error>` - The certificate record if found, or a database error
pub async fn get_cert(pool: &DbPool, domain: &str) -> Result<Option<CertRow>, sqlx::Error> {
    sqlx::query_as::<_, CertRow>("SELECT * FROM certs WHERE domain = ?")
        .bind(domain)
        .fetch_optional(pool)
        .await
}

/// Retrieves all certificate records from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<CertRow>, sqlx::Error>` - A list of all certificates or a database error
pub async fn get_all_certs(pool: &DbPool) -> Result<Vec<CertRow>, sqlx::Error> {
    sqlx::query_as::<_, CertRow>("SELECT * FROM certs ORDER BY expires_at ASC")
        .fetch_all(pool)
        .await
}

/// Retrieves certificates that are expiring before the specified threshold.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `threshold` - Expiration threshold timestamp (Unix epoch)
///
/// # Returns
/// * `Result<Vec<(String, Option<i64>)>, sqlx::Error>` - A list of expiring (domain, provider_id) tuples or a database error
pub async fn get_expiring_certs(
    pool: &DbPool,
    threshold: i64,
) -> Result<Vec<(String, Option<i64>)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CertRow>("SELECT * FROM certs WHERE expires_at < ?")
        .bind(threshold)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| (r.domain, r.provider_id))
        .collect())
}

/// Creates a new DNS provider for wildcard certificate renewal.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `name` - Descriptive name for the provider
/// * `provider_type` - Type of provider (e.g., 'cloudflare')
/// * `credentials` - JSON-encoded credentials for the provider
///
/// # Returns
/// * `Result<i64, sqlx::Error>` - The ID of the newly created provider or a database error
pub async fn create_dns_provider(
    pool: &DbPool,
    name: &str,
    provider_type: &str,
    credentials: &str,
) -> Result<i64, sqlx::Error> {
    let id = sqlx::query(
        "INSERT INTO dns_providers (name, provider_type, credentials) VALUES (?, ?, ?)",
    )
    .bind(name)
    .bind(provider_type)
    .bind(credentials)
    .execute(pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

/// Retrieves all DNS providers from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<DnsProviderRow>, sqlx::Error>` - A list of all DNS providers or a database error
pub async fn get_all_dns_providers(pool: &DbPool) -> Result<Vec<DnsProviderRow>, sqlx::Error> {
    sqlx::query_as::<_, DnsProviderRow>("SELECT * FROM dns_providers ORDER BY id DESC")
        .fetch_all(pool)
        .await
}

/// Retrieves a specific DNS provider by its ID.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `id` - ID of the DNS provider
///
/// # Returns
/// * `Result<Option<DnsProviderRow>, sqlx::Error>` - The DNS provider record if found, or a database error
pub async fn get_dns_provider(
    pool: &DbPool,
    id: i64,
) -> Result<Option<DnsProviderRow>, sqlx::Error> {
    sqlx::query_as::<_, DnsProviderRow>("SELECT * FROM dns_providers WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Deletes a DNS provider by its ID.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `id` - ID of the DNS provider to delete
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn delete_dns_provider(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM dns_providers WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
