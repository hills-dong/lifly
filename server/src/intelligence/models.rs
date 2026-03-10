use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Database models ────────────────────────────────────────────────────────

/// Reminder row from the `reminders` table.
#[derive(Debug, Clone, FromRow)]
pub struct Reminder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub data_object_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub trigger_at: DateTime<Utc>,
    pub repeat_rule: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Request / Response DTOs ────────────────────────────────────────────────

/// Public reminder representation returned by the API.
#[derive(Debug, Clone, Serialize)]
pub struct ReminderResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub data_object_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub trigger_at: DateTime<Utc>,
    pub repeat_rule: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Reminder> for ReminderResponse {
    fn from(r: Reminder) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            data_object_id: r.data_object_id,
            title: r.title,
            description: r.description,
            trigger_at: r.trigger_at,
            repeat_rule: r.repeat_rule,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// Request body for `POST /api/reminders`.
#[derive(Debug, Deserialize)]
pub struct CreateReminderRequest {
    pub title: String,
    pub description: Option<String>,
    pub trigger_at: DateTime<Utc>,
    pub repeat_rule: Option<serde_json::Value>,
    pub data_object_id: Option<Uuid>,
}

/// Request body for `PUT /api/reminders/:id`.
#[derive(Debug, Deserialize)]
pub struct UpdateReminderRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub trigger_at: Option<DateTime<Utc>>,
    pub repeat_rule: Option<serde_json::Value>,
}

/// Query parameters for `GET /api/reminders`.
#[derive(Debug, Deserialize)]
pub struct ReminderQuery {
    pub status: Option<String>,
}
