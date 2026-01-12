use super::DbPool;

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

/// Retrieves all access lists from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<AccessListRow>, sqlx::Error>` - A list of all access lists or a database error
pub async fn get_all_access_lists(pool: &DbPool) -> Result<Vec<AccessListRow>, sqlx::Error> {
    sqlx::query_as::<_, AccessListRow>("SELECT * FROM access_lists")
        .fetch_all(pool)
        .await
}

/// Retrieves all basic auth clients for all access lists.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<AccessListClientRow>, sqlx::Error>` - A list of all access list clients or a database error
pub async fn get_access_list_clients(
    pool: &DbPool,
) -> Result<Vec<AccessListClientRow>, sqlx::Error> {
    sqlx::query_as::<_, AccessListClientRow>("SELECT * FROM access_list_clients")
        .fetch_all(pool)
        .await
}

/// Retrieves all IP restrictions for all access lists.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<AccessListIpRow>, sqlx::Error>` - A list of all access list IP restrictions or a database error
pub async fn get_access_list_ips(pool: &DbPool) -> Result<Vec<AccessListIpRow>, sqlx::Error> {
    sqlx::query_as::<_, AccessListIpRow>("SELECT * FROM access_list_ips")
        .fetch_all(pool)
        .await
}

/// Creates a new access list.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `name` - Descriptive name for the access list
///
/// # Returns
/// * `Result<i64, sqlx::Error>` - The ID of the newly created access list or a database error
pub async fn create_access_list(pool: &DbPool, name: &str) -> Result<i64, sqlx::Error> {
    let id = sqlx::query("INSERT INTO access_lists (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

/// Deletes an access list by its ID.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `id` - ID of the access list to delete
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn delete_access_list(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM access_lists WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Adds a basic auth client to an access list.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `list_id` - ID of the access list
/// * `username` - Basic auth username
/// * `password_hash` - Hashed basic auth password
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn add_access_list_client(
    pool: &DbPool,
    list_id: i64,
    username: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO access_list_clients (list_id, username, password_hash) VALUES (?, ?, ?)",
    )
    .bind(list_id)
    .bind(username)
    .bind(password_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Removes a basic auth client from an access list.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `list_id` - ID of the access list
/// * `username` - Username to remove
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn remove_access_list_client(
    pool: &DbPool,
    list_id: i64,
    username: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM access_list_clients WHERE list_id = ? AND username = ?")
        .bind(list_id)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

/// Adds an IP restriction to an access list.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `list_id` - ID of the access list
/// * `ip` - IP address or CIDR range
/// * `action` - Action to take ('allow' or 'deny')
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn add_access_list_ip(
    pool: &DbPool,
    list_id: i64,
    ip: &str,
    action: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO access_list_ips (list_id, ip_address, action) VALUES (?, ?, ?)")
        .bind(list_id)
        .bind(ip)
        .bind(action)
        .execute(pool)
        .await?;
    Ok(())
}

/// Removes an IP restriction from an access list.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `list_id` - ID of the access list
/// * `ip` - IP address or CIDR range to remove
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn remove_access_list_ip(
    pool: &DbPool,
    list_id: i64,
    ip: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM access_list_ips WHERE list_id = ? AND ip_address = ?")
        .bind(list_id)
        .bind(ip)
        .execute(pool)
        .await?;
    Ok(())
}
