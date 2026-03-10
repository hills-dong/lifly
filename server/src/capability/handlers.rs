use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::Router;
use uuid::Uuid;

use crate::common::{ApiResponse, AppError, AppResult};
use crate::common::AppState;

use super::models::{CapabilityDetailResponse, CapabilityResponse, ListCapabilitiesQuery};
use super::repo;

/// `GET /api/capabilities?category=collect`
///
/// Returns all capabilities, optionally filtered by category.
async fn list_capabilities_handler(
    State(state): State<AppState>,
    Query(query): Query<ListCapabilitiesQuery>,
) -> AppResult<ApiResponse<Vec<CapabilityResponse>>> {
    let capabilities = repo::list_capabilities(&state.pool, query.category.as_deref()).await?;

    let items: Vec<CapabilityResponse> = capabilities.into_iter().map(Into::into).collect();

    Ok(ApiResponse::success(items))
}

/// `GET /api/capabilities/:id`
///
/// Returns a single capability together with its params.
async fn get_capability_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<CapabilityDetailResponse>> {
    let capability = repo::find_capability_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("capability {id} not found")))?;

    let params = repo::list_params_by_capability(&state.pool, id).await?;

    Ok(ApiResponse::success(capability.into_detail(params)))
}

/// Build the capability sub-router.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/capabilities", get(list_capabilities_handler))
        .route("/api/capabilities/{id}", get(get_capability_handler))
}
