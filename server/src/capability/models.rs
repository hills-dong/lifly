use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Database models ─────────────────────────────────────────────────────────

/// A single atomic capability that the system can perform.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AtomicCapability {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// One of: collect, process, store, use.
    pub category: String,
    /// One of: builtin, script, remote_llm.
    pub runtime_type: String,
    /// Arbitrary JSON blob describing how to invoke the capability at runtime.
    pub runtime_config: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A typed parameter (input or output) belonging to an [`AtomicCapability`].
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CapabilityParam {
    pub id: Uuid,
    pub capability_id: Uuid,
    pub name: String,
    /// One of: input, output.
    pub direction: String,
    pub data_type: String,
    pub is_required: bool,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── API response types ──────────────────────────────────────────────────────

/// Lightweight representation returned when listing capabilities.
#[derive(Debug, Serialize)]
pub struct CapabilityResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub runtime_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Full representation returned when fetching a single capability by id.
#[derive(Debug, Serialize)]
pub struct CapabilityDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub runtime_type: String,
    pub runtime_config: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub params: Vec<CapabilityParam>,
}

// ── Conversions ─────────────────────────────────────────────────────────────

impl From<AtomicCapability> for CapabilityResponse {
    fn from(c: AtomicCapability) -> Self {
        Self {
            id: c.id,
            name: c.name,
            description: c.description,
            category: c.category,
            runtime_type: c.runtime_type,
            is_active: c.is_active,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

impl AtomicCapability {
    /// Combine this capability with its params into a detail response.
    pub fn into_detail(self, params: Vec<CapabilityParam>) -> CapabilityDetailResponse {
        CapabilityDetailResponse {
            id: self.id,
            name: self.name,
            description: self.description,
            category: self.category,
            runtime_type: self.runtime_type,
            runtime_config: self.runtime_config,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
            params,
        }
    }
}

// ── Query parameters ────────────────────────────────────────────────────────

/// Query parameters for the list-capabilities endpoint.
#[derive(Debug, Deserialize)]
pub struct ListCapabilitiesQuery {
    pub category: Option<String>,
}
