use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::Reminder;

/// List reminders for a user, optionally filtered by status.
pub async fn list_reminders(
    pool: &PgPool,
    user_id: Uuid,
    status_filter: Option<String>,
) -> Result<Vec<Reminder>, sqlx::Error> {
    match status_filter {
        Some(status) => {
            sqlx::query_as::<_, Reminder>(
                "SELECT id, user_id, data_object_id, title, description, trigger_at,
                        repeat_rule, status, created_at, updated_at
                 FROM reminders
                 WHERE user_id = $1 AND status = $2
                 ORDER BY trigger_at ASC",
            )
            .bind(user_id)
            .bind(status)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, Reminder>(
                "SELECT id, user_id, data_object_id, title, description, trigger_at,
                        repeat_rule, status, created_at, updated_at
                 FROM reminders
                 WHERE user_id = $1
                 ORDER BY trigger_at ASC",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
        }
    }
}

/// Find a single reminder by its primary key.
pub async fn find_reminder_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Reminder>, sqlx::Error> {
    sqlx::query_as::<_, Reminder>(
        "SELECT id, user_id, data_object_id, title, description, trigger_at,
                repeat_rule, status, created_at, updated_at
         FROM reminders
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Insert a new reminder and return the created row.
pub async fn create_reminder(
    pool: &PgPool,
    user_id: Uuid,
    data_object_id: Option<Uuid>,
    title: &str,
    description: Option<&str>,
    trigger_at: DateTime<Utc>,
    repeat_rule: Option<&serde_json::Value>,
) -> Result<Reminder, sqlx::Error> {
    sqlx::query_as::<_, Reminder>(
        "INSERT INTO reminders (id, user_id, data_object_id, title, description, trigger_at,
                                repeat_rule, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', NOW(), NOW())
         RETURNING id, user_id, data_object_id, title, description, trigger_at,
                   repeat_rule, status, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(data_object_id)
    .bind(title)
    .bind(description)
    .bind(trigger_at)
    .bind(repeat_rule)
    .fetch_one(pool)
    .await
}

/// Update a reminder's mutable fields, returning the updated row.
pub async fn update_reminder(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
    trigger_at: Option<DateTime<Utc>>,
    repeat_rule: Option<&serde_json::Value>,
) -> Result<Reminder, sqlx::Error> {
    sqlx::query_as::<_, Reminder>(
        "UPDATE reminders
         SET title       = COALESCE($2, title),
             description = COALESCE($3, description),
             trigger_at  = COALESCE($4, trigger_at),
             repeat_rule = COALESCE($5, repeat_rule),
             updated_at  = NOW()
         WHERE id = $1
         RETURNING id, user_id, data_object_id, title, description, trigger_at,
                   repeat_rule, status, created_at, updated_at",
    )
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(trigger_at)
    .bind(repeat_rule)
    .fetch_one(pool)
    .await
}

/// Hard-delete a reminder by its primary key.
pub async fn delete_reminder(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM reminders WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Set a reminder's status to `dismissed`, returning the updated row.
pub async fn dismiss_reminder(pool: &PgPool, id: Uuid) -> Result<Reminder, sqlx::Error> {
    sqlx::query_as::<_, Reminder>(
        "UPDATE reminders
         SET status     = 'dismissed',
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, user_id, data_object_id, title, description, trigger_at,
                   repeat_rule, status, created_at, updated_at",
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

/// List all pending reminders whose `trigger_at` is before the given timestamp.
///
/// Useful for background jobs that fire reminder notifications.
pub async fn list_pending_reminders_before(
    pool: &PgPool,
    before: DateTime<Utc>,
) -> Result<Vec<Reminder>, sqlx::Error> {
    sqlx::query_as::<_, Reminder>(
        "SELECT id, user_id, data_object_id, title, description, trigger_at,
                repeat_rule, status, created_at, updated_at
         FROM reminders
         WHERE status = 'pending' AND trigger_at <= $1
         ORDER BY trigger_at ASC",
    )
    .bind(before)
    .fetch_all(pool)
    .await
}
