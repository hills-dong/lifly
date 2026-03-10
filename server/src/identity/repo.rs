use sqlx::PgPool;
use uuid::Uuid;

use super::models::{Device, User};

/// Find a user by their unique username.
pub async fn find_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, display_name, preferences, created_at, updated_at
         FROM users
         WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

/// Find a user by their primary key.
pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, display_name, preferences, created_at, updated_at
         FROM users
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Update a user's display name and/or preferences, returning the updated row.
pub async fn update_user_profile(
    pool: &PgPool,
    id: Uuid,
    display_name: Option<&str>,
    preferences: Option<&serde_json::Value>,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "UPDATE users
         SET display_name = COALESCE($2, display_name),
             preferences  = COALESCE($3, preferences),
             updated_at   = NOW()
         WHERE id = $1
         RETURNING id, username, password_hash, display_name, preferences, created_at, updated_at",
    )
    .bind(id)
    .bind(display_name)
    .bind(preferences)
    .fetch_one(pool)
    .await
}

/// List all devices belonging to a user.
pub async fn list_devices(pool: &PgPool, user_id: Uuid) -> Result<Vec<Device>, sqlx::Error> {
    sqlx::query_as::<_, Device>(
        "SELECT id, user_id, name, device_type, platform, token, is_active, last_seen_at, created_at, updated_at
         FROM devices
         WHERE user_id = $1
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Register a new device for a user.
pub async fn create_device(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    device_type: &str,
    platform: Option<&str>,
    token: Option<&str>,
) -> Result<Device, sqlx::Error> {
    sqlx::query_as::<_, Device>(
        "INSERT INTO devices (id, user_id, name, device_type, platform, token, is_active, last_seen_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, TRUE, NOW(), NOW(), NOW())
         RETURNING id, user_id, name, device_type, platform, token, is_active, last_seen_at, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(name)
    .bind(device_type)
    .bind(platform)
    .bind(token)
    .fetch_one(pool)
    .await
}
