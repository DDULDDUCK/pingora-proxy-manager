use super::DbPool;
use serde::Serialize;

#[derive(sqlx::FromRow, Debug, Clone, Serialize)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: i64,
    pub last_login: Option<i64>,
}

/// Retrieves the ID and password hash of a user by username.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `username` - Username to search for
///
/// # Returns
/// * `Result<Option<(i64, String)>, sqlx::Error>` - A tuple of (id, password_hash) if found, or a database error
pub async fn get_user(pool: &DbPool, username: &str) -> Result<Option<(i64, String)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>("SELECT id, password_hash FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

/// Retrieves full user information by username.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `username` - Username to search for
///
/// # Returns
/// * `Result<Option<UserRow>, sqlx::Error>` - The user record if found, or a database error
pub async fn get_user_full(pool: &DbPool, username: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>("SELECT id, username, password_hash, role, created_at, last_login FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

/// Retrieves all users from the database.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<Vec<UserRow>, sqlx::Error>` - A list of all users or a database error
pub async fn get_all_users(pool: &DbPool) -> Result<Vec<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, role, created_at, last_login FROM users ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

/// Creates a new user with the default 'admin' role, ignoring if it already exists.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `username` - New username
/// * `password_hash` - Hashed password
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn create_user(
    pool: &DbPool,
    username: &str,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO users (username, password_hash, role) VALUES (?, ?, 'admin')",
    )
    .bind(username)
    .bind(password_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Creates a new user with a specified role.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `username` - New username
/// * `password_hash` - Hashed password
/// * `role` - User role (e.g., 'admin', 'user')
///
/// # Returns
/// * `Result<i64, sqlx::Error>` - The ID of the newly created user or a database error
pub async fn create_user_with_role(
    pool: &DbPool,
    username: &str,
    password_hash: &str,
    role: &str,
) -> Result<i64, sqlx::Error> {
    let id = sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)")
        .bind(username)
        .bind(password_hash)
        .bind(role)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

/// Updates a user's password.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user
/// * `password_hash` - New hashed password
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn update_user_password(
    pool: &DbPool,
    user_id: i64,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(password_hash)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Updates a user's role.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user
/// * `role` - New role
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn update_user_role(pool: &DbPool, user_id: i64, role: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(role)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Deletes a user by their ID.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user to delete
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
pub async fn delete_user(pool: &DbPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Updates the last login timestamp for a user.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user
///
/// # Returns
/// * `Result<(), sqlx::Error>` - Success or a database error
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
