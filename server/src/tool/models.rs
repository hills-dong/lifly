use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Database models ────────────────────────────────────────────────────────

/// Tool row from the `tools` table.
#[derive(Debug, Clone, FromRow)]
pub struct Tool {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub status: String,
    pub data_schema: Option<serde_json::Value>,
    pub trigger_config: Option<serde_json::Value>,
    pub current_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// ToolVersion row from the `tool_versions` table.
#[derive(Debug, Clone, FromRow)]
pub struct ToolVersion {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub version_number: i32,
    pub change_log: Option<String>,
    pub data_schema_snapshot: Option<serde_json::Value>,
    pub creator_type: String,
    pub created_at: DateTime<Utc>,
}

/// ToolStep row from the `tool_steps` table.
#[derive(Debug, Clone, FromRow)]
pub struct ToolStep {
    pub id: Uuid,
    pub tool_version_id: Uuid,
    pub capability_id: Uuid,
    pub step_order: i32,
    pub input_mapping: Option<serde_json::Value>,
    pub output_mapping: Option<serde_json::Value>,
    pub condition: Option<serde_json::Value>,
    pub on_failure: String,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
}

/// RawInput row from the `raw_inputs` table.
#[derive(Debug, Clone, FromRow)]
pub struct RawInput {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Option<Uuid>,
    pub input_type: String,
    pub raw_content: String,
    pub metadata: Option<serde_json::Value>,
    pub processing_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pipeline row from the `pipelines` table.
#[derive(Debug, Clone, FromRow)]
pub struct Pipeline {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub tool_version_id: Uuid,
    pub raw_input_id: Uuid,
    pub status: String,
    pub context: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// StepExecution row from the `step_executions` table.
#[derive(Debug, Clone, FromRow)]
pub struct StepExecution {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub tool_step_id: Uuid,
    pub status: String,
    pub actual_input: Option<serde_json::Value>,
    pub actual_output: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Atomic capability (referenced by executor) ─────────────────────────────

/// Minimal representation of an atomic capability, loaded for step execution.
#[derive(Debug, Clone, FromRow)]
pub struct AtomicCapability {
    pub id: Uuid,
    pub name: String,
    pub runtime_type: String,
    pub runtime_config: Option<serde_json::Value>,
}

// ── Request / Response DTOs ────────────────────────────────────────────────

/// Public tool representation returned by the API.
#[derive(Debug, Serialize)]
pub struct ToolResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub status: String,
    pub current_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Tool> for ToolResponse {
    fn from(t: Tool) -> Self {
        Self {
            id: t.id,
            name: t.name,
            description: t.description,
            source: t.source,
            status: t.status,
            current_version_id: t.current_version_id,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

/// Detailed tool representation including schema and trigger config.
#[derive(Debug, Serialize)]
pub struct ToolDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub status: String,
    pub data_schema: Option<serde_json::Value>,
    pub trigger_config: Option<serde_json::Value>,
    pub current_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Tool> for ToolDetailResponse {
    fn from(t: Tool) -> Self {
        Self {
            id: t.id,
            name: t.name,
            description: t.description,
            source: t.source,
            status: t.status,
            data_schema: t.data_schema,
            trigger_config: t.trigger_config,
            current_version_id: t.current_version_id,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

/// Public version representation.
#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub version_number: i32,
    pub change_log: Option<String>,
    pub creator_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<ToolVersion> for VersionResponse {
    fn from(v: ToolVersion) -> Self {
        Self {
            id: v.id,
            tool_id: v.tool_id,
            version_number: v.version_number,
            change_log: v.change_log,
            creator_type: v.creator_type,
            created_at: v.created_at,
        }
    }
}

/// Detailed version representation including steps.
#[derive(Debug, Serialize)]
pub struct VersionDetailResponse {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub version_number: i32,
    pub change_log: Option<String>,
    pub data_schema_snapshot: Option<serde_json::Value>,
    pub creator_type: String,
    pub created_at: DateTime<Utc>,
    pub steps: Vec<StepResponse>,
}

/// Public step representation.
#[derive(Debug, Serialize)]
pub struct StepResponse {
    pub id: Uuid,
    pub capability_id: Uuid,
    pub step_order: i32,
    pub input_mapping: Option<serde_json::Value>,
    pub output_mapping: Option<serde_json::Value>,
    pub condition: Option<serde_json::Value>,
    pub on_failure: String,
    pub retry_count: i32,
}

impl From<ToolStep> for StepResponse {
    fn from(s: ToolStep) -> Self {
        Self {
            id: s.id,
            capability_id: s.capability_id,
            step_order: s.step_order,
            input_mapping: s.input_mapping,
            output_mapping: s.output_mapping,
            condition: s.condition,
            on_failure: s.on_failure,
            retry_count: s.retry_count,
        }
    }
}

/// Public raw input representation.
#[derive(Debug, Serialize)]
pub struct RawInputResponse {
    pub id: Uuid,
    pub input_type: String,
    pub raw_content: String,
    pub metadata: Option<serde_json::Value>,
    pub processing_status: String,
    pub pipeline_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for `POST /api/raw-inputs`.
#[derive(Debug, Deserialize)]
pub struct CreateRawInputRequest {
    pub device_id: Option<Uuid>,
    pub tool_id: Uuid,
    pub input_type: String,
    pub raw_content: String,
    pub metadata: Option<serde_json::Value>,
}

/// Public pipeline representation.
#[derive(Debug, Serialize)]
pub struct PipelineResponse {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub tool_version_id: Uuid,
    pub raw_input_id: Uuid,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Pipeline> for PipelineResponse {
    fn from(p: Pipeline) -> Self {
        Self {
            id: p.id,
            tool_id: p.tool_id,
            tool_version_id: p.tool_version_id,
            raw_input_id: p.raw_input_id,
            status: p.status,
            started_at: p.started_at,
            completed_at: p.completed_at,
            error_message: p.error_message,
            created_at: p.created_at,
        }
    }
}

/// Detailed pipeline representation including step executions.
#[derive(Debug, Serialize)]
pub struct PipelineDetailResponse {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub tool_version_id: Uuid,
    pub raw_input_id: Uuid,
    pub status: String,
    pub context: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub step_executions: Vec<StepExecutionResponse>,
}

/// Public step execution representation.
#[derive(Debug, Serialize)]
pub struct StepExecutionResponse {
    pub id: Uuid,
    pub tool_step_id: Uuid,
    pub status: String,
    pub actual_input: Option<serde_json::Value>,
    pub actual_output: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<StepExecution> for StepExecutionResponse {
    fn from(se: StepExecution) -> Self {
        Self {
            id: se.id,
            tool_step_id: se.tool_step_id,
            status: se.status,
            actual_input: se.actual_input,
            actual_output: se.actual_output,
            started_at: se.started_at,
            completed_at: se.completed_at,
            duration_ms: se.duration_ms,
            error_message: se.error_message,
            created_at: se.created_at,
        }
    }
}
