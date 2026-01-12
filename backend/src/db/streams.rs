use super::DbPool;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct StreamRow {
    pub id: i64,
    pub listen_port: i64,
    pub forward_host: String,
    pub forward_port: i64,
    pub protocol: String,
}

/// Retrieves all configured L4 streams from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<StreamRow>, sqlx::Error>` - A list of all streams or a database error
pub async fn get_all_streams(pool: &DbPool) -> Result<Vec<StreamRow>, sqlx::Error> {
    sqlx::query_as::<_, StreamRow>("SELECT * FROM streams")
        .fetch_all(pool)
        .await
}

/// Inserts or updates an L4 stream configuration.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `listen_port` - Port to listen on
/// * `forward_host` - Upstream host to forward traffic to
/// * `forward_port` - Upstream port to forward traffic to
/// * `protocol` - Protocol to use ('tcp' or 'udp')
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
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

/// Deletes a stream configuration by its listen port.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `listen_port` - Listen port of the stream to delete
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn delete_stream(pool: &DbPool, listen_port: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM streams WHERE listen_port = ?")
        .bind(listen_port)
        .execute(pool)
        .await?;
    Ok(())
}
