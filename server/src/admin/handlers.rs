//! Generic CRUD + metadata handlers, dispatched by `{resource}` against the
//! [`registry`](super::registry).

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use serde_json::json;

use crate::common::{AdminUser, ApiResponse, AppError, AppResult, AppState};

use super::registry;
use super::repo;

const DEFAULT_PER_PAGE: i64 = 20;
const MAX_PER_PAGE: i64 = 200;
const RESERVED_QUERY_KEYS: [&str; 4] = ["page", "per_page", "sort", "order"];

/// Resolve a resource spec or return a 404.
fn resolve(resource: &str) -> AppResult<&'static registry::ResourceSpec> {
    registry::find(resource).ok_or_else(|| AppError::NotFound(format!("unknown resource '{resource}'")))
}

/// `GET /api/admin/meta` — the full resource registry for dynamic UIs.
pub async fn meta_handler(_admin: AdminUser) -> AppResult<ApiResponse<serde_json::Value>> {
    let resources = serde_json::to_value(registry::all())
        .map_err(|e| AppError::Internal(format!("failed to serialize registry: {e}")))?;
    Ok(ApiResponse::success(json!({ "resources": resources })))
}

/// `GET /api/admin/data/{resource}` — paginated, filtered, sorted list.
pub async fn list_handler(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(resource): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    let spec = resolve(&resource)?;

    let page = params
        .get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|p| *p >= 1)
        .unwrap_or(1);
    let per_page = params
        .get("per_page")
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|p| *p >= 1)
        .unwrap_or(DEFAULT_PER_PAGE)
        .min(MAX_PER_PAGE);

    // Default sort: created_at if present, else the primary key.
    let default_sort = if spec.column("created_at").is_some() {
        "created_at"
    } else {
        spec.pk
    };
    let sort = params
        .get("sort")
        .filter(|s| spec.column(s).is_some())
        .map(|s| s.as_str())
        .unwrap_or(default_sort);
    let descending = !params
        .get("order")
        .map(|o| o.eq_ignore_ascii_case("asc"))
        .unwrap_or(false);

    // Remaining params are equality filters; each must be a real column.
    let mut filters = Vec::new();
    for (k, v) in &params {
        if RESERVED_QUERY_KEYS.contains(&k.as_str()) {
            continue;
        }
        if spec.column(k).is_none() {
            return Err(AppError::Validation(format!("unknown filter field '{k}'")));
        }
        filters.push((k.clone(), v.clone()));
    }

    let offset = (page - 1) * per_page;
    let (items, total) = repo::list(&state.pool, spec, &filters, sort, descending, per_page, offset).await?;

    Ok(ApiResponse::success(json!({
        "items": items,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

/// `GET /api/admin/data/{resource}/{id}` — single row.
pub async fn get_one_handler(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path((resource, id)): Path<(String, String)>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    let spec = resolve(&resource)?;
    let row = repo::get(&state.pool, spec, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("{resource} {id} not found")))?;
    Ok(ApiResponse::success(row))
}

/// `POST /api/admin/data/{resource}` — create a row.
pub async fn create_handler(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(resource): Path<String>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    let spec = resolve(&resource)?;
    let row = repo::create(&state.pool, spec, &body).await?;
    Ok(ApiResponse::success(row))
}

/// `PUT /api/admin/data/{resource}/{id}` — update a row.
pub async fn update_handler(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path((resource, id)): Path<(String, String)>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    let spec = resolve(&resource)?;
    let row = repo::update(&state.pool, spec, &id, &body)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("{resource} {id} not found")))?;
    Ok(ApiResponse::success(row))
}

/// `DELETE /api/admin/data/{resource}/{id}` — delete a row.
pub async fn delete_one_handler(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path((resource, id)): Path<(String, String)>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    let spec = resolve(&resource)?;
    let deleted = repo::delete(&state.pool, spec, &id).await?;
    if !deleted {
        return Err(AppError::NotFound(format!("{resource} {id} not found")));
    }
    Ok(ApiResponse::success(json!({ "id": id })))
}
