use axum::extract::{Json, Path, Query, State};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use uuid::Uuid;

use crate::common::{ApiResponse, AppError, AppResult, AppState, AuthUser};

use super::models::{
    CreateRawInputRequest, PipelineDetailResponse, PipelineResponse, RawInputResponse,
    StepExecutionResponse, StepResponse, ToolDetailResponse, ToolResponse, VersionDetailResponse,
    VersionResponse,
};
use super::pipeline::engine::PipelineEngine;
use super::repo;

// ── Tool handlers ──────────────────────────────────────────────────────────

/// GET /api/tools
async fn list_tools_handler(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<ApiResponse<Vec<ToolResponse>>> {
    let tools = repo::list_tools(&state.pool, auth.user_id).await?;
    let resp: Vec<ToolResponse> = tools.into_iter().map(Into::into).collect();
    Ok(ApiResponse::success(resp))
}

/// GET /api/tools/:id
async fn get_tool_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<ToolDetailResponse>> {
    let tool = repo::find_tool_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tool {id} not found")))?;

    // Ensure the requesting user owns this tool.
    if tool.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("tool {id} not found")));
    }

    Ok(ApiResponse::success(ToolDetailResponse::from(tool)))
}

// ── Version handlers ───────────────────────────────────────────────────────

/// GET /api/tools/:id/versions
async fn list_versions_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(tool_id): Path<Uuid>,
) -> AppResult<ApiResponse<Vec<VersionResponse>>> {
    // Verify tool ownership.
    let tool = repo::find_tool_by_id(&state.pool, tool_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tool {tool_id} not found")))?;
    if tool.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("tool {tool_id} not found")));
    }

    let versions = repo::list_versions(&state.pool, tool_id).await?;
    let resp: Vec<VersionResponse> = versions.into_iter().map(Into::into).collect();
    Ok(ApiResponse::success(resp))
}

/// GET /api/tools/:id/versions/:vid
async fn get_version_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tool_id, version_id)): Path<(Uuid, Uuid)>,
) -> AppResult<ApiResponse<VersionDetailResponse>> {
    // Verify tool ownership.
    let tool = repo::find_tool_by_id(&state.pool, tool_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tool {tool_id} not found")))?;
    if tool.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("tool {tool_id} not found")));
    }

    let version = repo::find_version_by_id(&state.pool, version_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("version {version_id} not found")))?;

    // Ensure this version belongs to the requested tool.
    if version.tool_id != tool_id {
        return Err(AppError::NotFound(format!("version {version_id} not found")));
    }

    let steps = repo::list_steps_by_version(&state.pool, version_id).await?;
    let step_responses: Vec<StepResponse> = steps.into_iter().map(Into::into).collect();

    Ok(ApiResponse::success(VersionDetailResponse {
        id: version.id,
        tool_id: version.tool_id,
        version_number: version.version_number,
        change_log: version.change_log,
        data_schema_snapshot: version.data_schema_snapshot,
        creator_type: version.creator_type,
        created_at: version.created_at,
        steps: step_responses,
    }))
}

// ── Raw input handlers ─────────────────────────────────────────────────────

/// POST /api/raw-inputs
async fn create_raw_input_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateRawInputRequest>,
) -> AppResult<ApiResponse<RawInputResponse>> {
    // Validate input_type.
    let valid_types = ["text", "image", "audio", "video", "url"];
    if !valid_types.contains(&body.input_type.as_str()) {
        return Err(AppError::Validation(format!(
            "invalid input_type '{}', expected one of: {}",
            body.input_type,
            valid_types.join(", ")
        )));
    }

    // Verify the tool exists and belongs to the user.
    let tool = repo::find_tool_by_id(&state.pool, body.tool_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tool {} not found", body.tool_id)))?;
    if tool.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("tool {} not found", body.tool_id)));
    }

    // The tool must have a current version to execute.
    let version_id = tool.current_version_id.ok_or_else(|| {
        AppError::Validation(format!("tool {} has no active version", body.tool_id))
    })?;

    // Create the raw input.
    let raw_input = repo::create_raw_input(
        &state.pool,
        auth.user_id,
        body.device_id,
        &body.input_type,
        &body.raw_content,
        body.metadata.as_ref(),
    )
    .await?;

    // Create the pipeline.
    let pipeline =
        repo::create_pipeline(&state.pool, body.tool_id, version_id, raw_input.id).await?;

    // Update raw input status to processing.
    repo::update_raw_input_status(&state.pool, raw_input.id, "processing").await?;

    // Spawn pipeline execution in the background.
    let engine = PipelineEngine::new(state.pool.clone(), state.config.clone());
    let pipeline_id = pipeline.id;
    tokio::spawn(async move {
        if let Err(e) = engine.execute(pipeline_id).await {
            tracing::error!(pipeline_id = %pipeline_id, error = %e, "pipeline execution failed");
        }
    });

    Ok(ApiResponse::success(RawInputResponse {
        id: raw_input.id,
        input_type: raw_input.input_type,
        raw_content: raw_input.raw_content,
        metadata: raw_input.metadata,
        processing_status: "processing".to_string(),
        pipeline_id: Some(pipeline.id),
        created_at: raw_input.created_at,
        updated_at: raw_input.updated_at,
    }))
}

/// GET /api/raw-inputs/:id
async fn get_raw_input_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<RawInputResponse>> {
    let raw_input = repo::find_raw_input_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("raw input {id} not found")))?;

    if raw_input.user_id != auth.user_id {
        return Err(AppError::NotFound(format!("raw input {id} not found")));
    }

    // Find associated pipeline if any.
    let pipelines = repo::list_pipelines(&state.pool, None, None).await?;
    let pipeline_id = pipelines
        .iter()
        .find(|p| p.raw_input_id == raw_input.id)
        .map(|p| p.id);

    Ok(ApiResponse::success(RawInputResponse {
        id: raw_input.id,
        input_type: raw_input.input_type,
        raw_content: raw_input.raw_content,
        metadata: raw_input.metadata,
        processing_status: raw_input.processing_status,
        pipeline_id,
        created_at: raw_input.created_at,
        updated_at: raw_input.updated_at,
    }))
}

// ── Pipeline handlers ──────────────────────────────────────────────────────

/// Query parameters for listing pipelines.
#[derive(Debug, Deserialize)]
pub struct ListPipelinesQuery {
    pub tool_id: Option<Uuid>,
    pub status: Option<String>,
}

/// GET /api/pipelines
async fn list_pipelines_handler(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ListPipelinesQuery>,
) -> AppResult<ApiResponse<Vec<PipelineResponse>>> {
    let pipelines = repo::list_pipelines(
        &state.pool,
        query.tool_id,
        query.status.as_deref(),
    )
    .await?;
    let resp: Vec<PipelineResponse> = pipelines.into_iter().map(Into::into).collect();
    Ok(ApiResponse::success(resp))
}

/// GET /api/pipelines/:id
async fn get_pipeline_handler(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<PipelineDetailResponse>> {
    let pipeline = repo::find_pipeline_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("pipeline {id} not found")))?;

    let step_executions = repo::list_step_executions(&state.pool, id).await?;
    let step_responses: Vec<StepExecutionResponse> =
        step_executions.into_iter().map(Into::into).collect();

    Ok(ApiResponse::success(PipelineDetailResponse {
        id: pipeline.id,
        tool_id: pipeline.tool_id,
        tool_version_id: pipeline.tool_version_id,
        raw_input_id: pipeline.raw_input_id,
        status: pipeline.status,
        context: pipeline.context,
        started_at: pipeline.started_at,
        completed_at: pipeline.completed_at,
        error_message: pipeline.error_message,
        created_at: pipeline.created_at,
        step_executions: step_responses,
    }))
}

// ── Router ─────────────────────────────────────────────────────────────────

/// Build the router subtree for the tool module.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/tools", get(list_tools_handler))
        .route("/api/tools/{id}", get(get_tool_handler))
        .route("/api/tools/{id}/versions", get(list_versions_handler))
        .route(
            "/api/tools/{id}/versions/{vid}",
            get(get_version_handler),
        )
        .route("/api/raw-inputs", post(create_raw_input_handler))
        .route("/api/raw-inputs/{id}", get(get_raw_input_handler))
        .route("/api/pipelines", get(list_pipelines_handler))
        .route("/api/pipelines/{id}", get(get_pipeline_handler))
}
