use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Database models ────────────────────────────────────────────────────────

/// User row from the `users` table.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub preferences: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Device row from the `devices` table.
#[derive(Debug, Clone, FromRow)]
pub struct Device {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub device_type: String,
    pub platform: Option<String>,
    pub token: Option<String>,
    pub is_active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Request / Response DTOs ────────────────────────────────────────────────

/// Request body for `POST /api/auth/login`.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response body for `POST /api/auth/login`.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserProfile,
}

/// Public user profile (never exposes `password_hash`).
#[derive(Debug, Clone, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub preferences: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            preferences: u.preferences,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

/// Request body for `PUT /api/user/profile`.
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

/// Public device representation returned by the API.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceResponse {
    pub id: Uuid,
    pub name: String,
    pub device_type: String,
    pub platform: Option<String>,
    pub is_active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Device> for DeviceResponse {
    fn from(d: Device) -> Self {
        Self {
            id: d.id,
            name: d.name,
            device_type: d.device_type,
            platform: d.platform,
            is_active: d.is_active,
            last_seen_at: d.last_seen_at,
            created_at: d.created_at,
        }
    }
}

/// Request body for `POST /api/user/devices`.
#[derive(Debug, Deserialize)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub device_type: String,
    pub platform: Option<String>,
    pub token: Option<String>,
}
