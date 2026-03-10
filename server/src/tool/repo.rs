use sqlx::PgPool;
use uuid::Uuid;

use super::models::{
    AtomicCapability, Pipeline, RawInput, StepExecution, Tool, ToolStep, ToolVersion,
};

// ── Tool queries ───────────────────────────────────────────────────────────

/// List all tools belonging to a user.
pub async fn list_tools(pool: &PgPool, user_id: Uuid) -> Result<Vec<Tool>, sqlx::Error> {
    sqlx::query_as::<_, Tool>(
        "SELECT id, user_id, name, description, source, status,
                data_schema, trigger_config, current_version_id,
                created_at, updated_at
         FROM tools
         WHERE user_id = $1
         ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Find a tool by its primary key.
pub async fn find_tool_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Tool>, sqlx::Error> {
    sqlx::query_as::<_, Tool>(
        "SELECT id, user_id, name, description, source, status,
                data_schema, trigger_config, current_version_id,
                created_at, updated_at
         FROM tools
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// ── Version queries ────────────────────────────────────────────────────────

/// List all versions for a given tool.
pub async fn list_versions(
    pool: &PgPool,
    tool_id: Uuid,
) -> Result<Vec<ToolVersion>, sqlx::Error> {
    sqlx::query_as::<_, ToolVersion>(
        "SELECT id, tool_id, version_number, change_log,
                data_schema_snapshot, creator_type, created_at
         FROM tool_versions
         WHERE tool_id = $1
         ORDER BY version_number DESC",
    )
    .bind(tool_id)
    .fetch_all(pool)
    .await
}

/// Find a specific tool version by its primary key.
pub async fn find_version_by_id(
    pool: &PgPool,
    version_id: Uuid,
) -> Result<Option<ToolVersion>, sqlx::Error> {
    sqlx::query_as::<_, ToolVersion>(
        "SELECT id, tool_id, version_number, change_log,
                data_schema_snapshot, creator_type, created_at
         FROM tool_versions
         WHERE id = $1",
    )
    .bind(version_id)
    .fetch_optional(pool)
    .await
}

// ── Step queries ───────────────────────────────────────────────────────────

/// List all steps for a given tool version, ordered by step_order.
pub async fn list_steps_by_version(
    pool: &PgPool,
    version_id: Uuid,
) -> Result<Vec<ToolStep>, sqlx::Error> {
    sqlx::query_as::<_, ToolStep>(
        "SELECT id, tool_version_id, capability_id, step_order,
                input_mapping, output_mapping, condition,
                on_failure, retry_count, created_at
         FROM tool_steps
         WHERE tool_version_id = $1
         ORDER BY step_order ASC",
    )
    .bind(version_id)
    .fetch_all(pool)
    .await
}

// ── Raw input queries ──────────────────────────────────────────────────────

/// Create a new raw input record.
pub async fn create_raw_input(
    pool: &PgPool,
    user_id: Uuid,
    device_id: Option<Uuid>,
    input_type: &str,
    raw_content: &str,
    metadata: Option<&serde_json::Value>,
) -> Result<RawInput, sqlx::Error> {
    sqlx::query_as::<_, RawInput>(
        "INSERT INTO raw_inputs (id, user_id, device_id, input_type, raw_content, metadata,
                                  processing_status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', NOW(), NOW())
         RETURNING id, user_id, device_id, input_type, raw_content, metadata,
                   processing_status, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(device_id)
    .bind(input_type)
    .bind(raw_content)
    .bind(metadata)
    .fetch_one(pool)
    .await
}

/// Find a raw input by its primary key.
pub async fn find_raw_input_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<RawInput>, sqlx::Error> {
    sqlx::query_as::<_, RawInput>(
        "SELECT id, user_id, device_id, input_type, raw_content, metadata,
                processing_status, created_at, updated_at
         FROM raw_inputs
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Update the processing status of a raw input.
pub async fn update_raw_input_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE raw_inputs
         SET processing_status = $2, updated_at = NOW()
         WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Pipeline queries ───────────────────────────────────────────────────────

/// Create a new pipeline record.
pub async fn create_pipeline(
    pool: &PgPool,
    tool_id: Uuid,
    version_id: Uuid,
    raw_input_id: Uuid,
) -> Result<Pipeline, sqlx::Error> {
    sqlx::query_as::<_, Pipeline>(
        "INSERT INTO pipelines (id, tool_id, tool_version_id, raw_input_id,
                                 status, context, created_at)
         VALUES ($1, $2, $3, $4, 'pending', '{}'::jsonb, NOW())
         RETURNING id, tool_id, tool_version_id, raw_input_id, status, context,
                   started_at, completed_at, error_message, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(tool_id)
    .bind(version_id)
    .bind(raw_input_id)
    .fetch_one(pool)
    .await
}

/// Update pipeline status, optionally setting error message and timestamps.
pub async fn update_pipeline_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    error_message: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE pipelines
         SET status = $2,
             error_message = $3,
             started_at = CASE WHEN $2 = 'running' AND started_at IS NULL THEN NOW() ELSE started_at END,
             completed_at = CASE WHEN $2 IN ('completed', 'failed') THEN NOW() ELSE completed_at END
         WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(error_message)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update the pipeline's context JSONB value.
pub async fn update_pipeline_context(
    pool: &PgPool,
    id: Uuid,
    context: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE pipelines SET context = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(context)
    .execute(pool)
    .await?;
    Ok(())
}

/// Find a pipeline by its primary key.
pub async fn find_pipeline_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Pipeline>, sqlx::Error> {
    sqlx::query_as::<_, Pipeline>(
        "SELECT id, tool_id, tool_version_id, raw_input_id, status, context,
                started_at, completed_at, error_message, created_at
         FROM pipelines
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// List pipelines with optional filters for tool_id and status.
pub async fn list_pipelines(
    pool: &PgPool,
    tool_id_filter: Option<Uuid>,
    status_filter: Option<&str>,
) -> Result<Vec<Pipeline>, sqlx::Error> {
    sqlx::query_as::<_, Pipeline>(
        "SELECT id, tool_id, tool_version_id, raw_input_id, status, context,
                started_at, completed_at, error_message, created_at
         FROM pipelines
         WHERE ($1::uuid IS NULL OR tool_id = $1)
           AND ($2::text IS NULL OR status = $2)
         ORDER BY created_at DESC",
    )
    .bind(tool_id_filter)
    .bind(status_filter)
    .fetch_all(pool)
    .await
}

// ── Step execution queries ─────────────────────────────────────────────────

/// Create a new step execution record.
pub async fn create_step_execution(
    pool: &PgPool,
    pipeline_id: Uuid,
    tool_step_id: Uuid,
) -> Result<StepExecution, sqlx::Error> {
    sqlx::query_as::<_, StepExecution>(
        "INSERT INTO step_executions (id, pipeline_id, tool_step_id, status, created_at)
         VALUES ($1, $2, $3, 'pending', NOW())
         RETURNING id, pipeline_id, tool_step_id, status, actual_input, actual_output,
                   started_at, completed_at, duration_ms, error_message, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(pipeline_id)
    .bind(tool_step_id)
    .fetch_one(pool)
    .await
}

/// Update a step execution with results.
pub async fn update_step_execution(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    actual_input: Option<&serde_json::Value>,
    actual_output: Option<&serde_json::Value>,
    duration_ms: Option<i32>,
    error_message: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE step_executions
         SET status = $2,
             actual_input = COALESCE($3, actual_input),
             actual_output = COALESCE($4, actual_output),
             started_at = CASE WHEN $2 = 'running' AND started_at IS NULL THEN NOW() ELSE started_at END,
             completed_at = CASE WHEN $2 IN ('completed', 'failed', 'skipped') THEN NOW() ELSE completed_at END,
             duration_ms = COALESCE($5, duration_ms),
             error_message = $6
         WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(actual_input)
    .bind(actual_output)
    .bind(duration_ms)
    .bind(error_message)
    .execute(pool)
    .await?;
    Ok(())
}

/// List all step executions for a pipeline.
pub async fn list_step_executions(
    pool: &PgPool,
    pipeline_id: Uuid,
) -> Result<Vec<StepExecution>, sqlx::Error> {
    sqlx::query_as::<_, StepExecution>(
        "SELECT id, pipeline_id, tool_step_id, status, actual_input, actual_output,
                started_at, completed_at, duration_ms, error_message, created_at
         FROM step_executions
         WHERE pipeline_id = $1
         ORDER BY created_at ASC",
    )
    .bind(pipeline_id)
    .fetch_all(pool)
    .await
}

// ── Capability lookup ──────────────────────────────────────────────────────

/// Load an atomic capability by its primary key.
pub async fn find_capability_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<AtomicCapability>, sqlx::Error> {
    sqlx::query_as::<_, AtomicCapability>(
        "SELECT id, name, runtime_type, runtime_config
         FROM atomic_capabilities
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
