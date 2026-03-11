use std::path::Path;

use axum::body::Body;
use axum::extract::{Multipart, Path as AxumPath, Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Json;
use axum::Router;
use chrono::Utc;
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::common::{ApiResponse, AppError, AppResult, AppState, AuthUser};

use super::models::{
    CategoryResponse, CreateCategoryRequest, DataObjectDetailResponse, DataObjectQuery,
    DataObjectResponse, FileStorageResponse, UpdateCategoryRequest, UpdateDataObjectRequest,
    UploadResponse,
};
use super::repo;

// ── Data-object handlers ───────────────────────────────────────────────────

/// GET /api/data-objects
async fn list_data_objects(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<DataObjectQuery>,
) -> AppResult<ApiResponse<Vec<DataObjectResponse>>> {
    let rows = repo::list_data_objects(&state.pool, &query).await?;
    let data = rows.into_iter().map(DataObjectResponse::from).collect();
    Ok(ApiResponse::success(data))
}

/// GET /api/data-objects/:id
async fn get_data_object(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<ApiResponse<DataObjectDetailResponse>> {
    let obj = repo::find_data_object_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("data object {id} not found")))?;

    let files = repo::list_files_by_data_object(&state.pool, id).await?;
    let file_responses: Vec<FileStorageResponse> =
        files.into_iter().map(FileStorageResponse::from).collect();

    let detail = DataObjectDetailResponse {
        id: obj.id,
        tool_id: obj.tool_id,
        pipeline_id: obj.pipeline_id,
        parent_id: obj.parent_id,
        category_id: obj.category_id,
        attributes: obj.attributes,
        status: obj.status,
        files: file_responses,
        created_at: obj.created_at,
        updated_at: obj.updated_at,
    };

    Ok(ApiResponse::success(detail))
}

/// PUT /api/data-objects/:id
async fn update_data_object(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath(id): AxumPath<Uuid>,
    Json(body): Json<UpdateDataObjectRequest>,
) -> AppResult<ApiResponse<DataObjectResponse>> {
    let updated = repo::update_data_object(
        &state.pool,
        id,
        body.attributes.as_ref(),
        body.category_id,
        body.status.as_deref(),
    )
    .await?;
    Ok(ApiResponse::success(DataObjectResponse::from(updated)))
}

/// DELETE /api/data-objects/:id
async fn delete_data_object(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<ApiResponse<()>> {
    repo::soft_delete_data_object(&state.pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// GET /api/data-objects/:id/files
async fn list_data_object_files(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<ApiResponse<Vec<FileStorageResponse>>> {
    // Verify the data object exists.
    repo::find_data_object_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("data object {id} not found")))?;

    let files = repo::list_files_by_data_object(&state.pool, id).await?;
    let data = files.into_iter().map(FileStorageResponse::from).collect();
    Ok(ApiResponse::success(data))
}

/// Query for search.
#[derive(Debug, serde::Deserialize)]
struct SearchQuery {
    q: Option<String>,
    tool_id: Option<Uuid>,
    limit: Option<i64>,
}

/// GET /api/data-objects/search — text search on attributes.
async fn search_data_objects(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<SearchQuery>,
) -> AppResult<ApiResponse<Vec<DataObjectResponse>>> {
    let q = query.q.unwrap_or_default();
    if q.is_empty() {
        return Ok(ApiResponse::success(vec![]));
    }
    let limit = query.limit.unwrap_or(50).min(200);
    let rows = repo::search_data_objects(&state.pool, &q, query.tool_id, limit).await?;
    let data = rows.into_iter().map(DataObjectResponse::from).collect();
    Ok(ApiResponse::success(data))
}

// ── File handlers ──────────────────────────────────────────────────────────

/// POST /api/files/upload
///
/// Accepts a `multipart/form-data` request with:
///   - `file` — the file part (required)
///   - `data_object_id` — optional UUID text field
///   - `raw_input_id` — optional UUID text field
///   - `role` — optional role string (default "original")
async fn upload_file(
    State(state): State<AppState>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<ApiResponse<UploadResponse>> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut original_file_name: Option<String> = None;
    let mut data_object_id: Option<Uuid> = None;
    let mut raw_input_id: Option<Uuid> = None;
    let mut role = "original".to_string();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart error: {e}")))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                original_file_name = field.file_name().map(|s| s.to_string());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("failed to read file: {e}")))?;
                file_bytes = Some(bytes.to_vec());
            }
            "data_object_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("invalid field: {e}")))?;
                if !text.is_empty() {
                    data_object_id = Some(
                        text.parse::<Uuid>()
                            .map_err(|_| AppError::Validation("invalid data_object_id".into()))?,
                    );
                }
            }
            "raw_input_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("invalid field: {e}")))?;
                if !text.is_empty() {
                    raw_input_id = Some(
                        text.parse::<Uuid>()
                            .map_err(|_| AppError::Validation("invalid raw_input_id".into()))?,
                    );
                }
            }
            "role" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("invalid field: {e}")))?;
                if !text.is_empty() {
                    role = text;
                }
            }
            _ => {
                // Skip unknown fields.
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| AppError::Validation("missing file field".into()))?;
    let orig_name = original_file_name.unwrap_or_else(|| "upload".to_string());

    // Determine extension from original name.
    let extension = Path::new(&orig_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin")
        .to_string();

    // Build storage path: {storage_path}/{year}/{month}/{uuid}.{ext}
    let now = Utc::now();
    let year = now.format("%Y");
    let month = now.format("%m");
    let file_uuid = Uuid::new_v4();
    let relative_path = format!("{year}/{month}/{file_uuid}.{extension}");
    let full_dir = state
        .config
        .file_storage_path
        .join(format!("{year}/{month}"));
    let full_path = state.config.file_storage_path.join(&relative_path);

    // Ensure directory exists.
    fs::create_dir_all(&full_dir)
        .await
        .map_err(|e| AppError::Internal(format!("failed to create directory: {e}")))?;

    // Compute SHA-256 checksum.
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let checksum = hex::encode(hasher.finalize());

    let file_size = bytes.len() as i64;

    // Write file to disk.
    let mut file = fs::File::create(&full_path)
        .await
        .map_err(|e| AppError::Internal(format!("failed to create file: {e}")))?;
    file.write_all(&bytes)
        .await
        .map_err(|e| AppError::Internal(format!("failed to write file: {e}")))?;
    file.flush()
        .await
        .map_err(|e| AppError::Internal(format!("failed to flush file: {e}")))?;

    // Guess MIME type from extension.
    let mime_type = mime_guess::from_path(&orig_name)
        .first_or_octet_stream()
        .to_string();

    // Create database record.
    let record = repo::create_file_storage(
        &state.pool,
        data_object_id,
        raw_input_id,
        &relative_path,
        &orig_name,
        &mime_type,
        file_size,
        &checksum,
        &role,
    )
    .await?;

    Ok(ApiResponse::success(UploadResponse::from(record)))
}

/// GET /api/files/:id — download / serve a stored file.
async fn download_file(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath(id): AxumPath<Uuid>,
) -> AppResult<impl IntoResponse> {
    let record = repo::find_file_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("file {id} not found")))?;

    let full_path = state.config.file_storage_path.join(&record.file_path);

    let file = fs::File::open(&full_path)
        .await
        .map_err(|e| AppError::Internal(format!("failed to open file: {e}")))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, record.mime_type),
        (
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", record.file_name),
        ),
    ];

    Ok((headers, body))
}

// ── Category handlers ──────────────────────────────────────────────────────

/// GET /api/tools/:tool_id/categories
async fn list_categories(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath(tool_id): AxumPath<Uuid>,
) -> AppResult<ApiResponse<Vec<CategoryResponse>>> {
    let rows = repo::list_categories(&state.pool, tool_id).await?;
    let data = rows.into_iter().map(CategoryResponse::from).collect();
    Ok(ApiResponse::success(data))
}

/// POST /api/tools/:tool_id/categories
async fn create_category(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath(tool_id): AxumPath<Uuid>,
    Json(body): Json<CreateCategoryRequest>,
) -> AppResult<ApiResponse<CategoryResponse>> {
    if body.name.is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }

    let cat = repo::create_category(
        &state.pool,
        tool_id,
        body.parent_id,
        &body.name,
        body.sort_order.unwrap_or(0),
    )
    .await?;

    Ok(ApiResponse::success(CategoryResponse::from(cat)))
}

/// PUT /api/tools/:tool_id/categories/:cid
async fn update_category(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath((_tool_id, cid)): AxumPath<(Uuid, Uuid)>,
    Json(body): Json<UpdateCategoryRequest>,
) -> AppResult<ApiResponse<CategoryResponse>> {
    let updated =
        repo::update_category(&state.pool, cid, body.name.as_deref(), body.sort_order).await?;
    Ok(ApiResponse::success(CategoryResponse::from(updated)))
}

/// DELETE /api/tools/:tool_id/categories/:cid
async fn delete_category(
    State(state): State<AppState>,
    _auth: AuthUser,
    AxumPath((_tool_id, cid)): AxumPath<(Uuid, Uuid)>,
) -> AppResult<ApiResponse<()>> {
    repo::delete_category(&state.pool, cid).await?;
    Ok(ApiResponse::success(()))
}

// ── Router constructors ────────────────────────────────────────────────────

/// Routes for data-object and file endpoints.
///
/// Mount points:
///   - `/api/data-objects/*`
///   - `/api/files/*`
pub fn routes() -> Router<AppState> {
    Router::new()
        // Data objects
        .route("/api/data-objects", get(list_data_objects))
        .route("/api/data-objects/search", get(search_data_objects))
        .route(
            "/api/data-objects/{id}",
            get(get_data_object)
                .put(update_data_object)
                .delete(delete_data_object),
        )
        .route("/api/data-objects/{id}/files", get(list_data_object_files))
        // Files
        .route("/api/files/upload", post(upload_file))
        .route("/api/files/{id}", get(download_file))
}

/// Routes for category endpoints, nested under `/api/tools/:tool_id`.
///
/// Mount points:
///   - `/api/tools/:tool_id/categories`
///   - `/api/tools/:tool_id/categories/:cid`
pub fn category_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/tools/{tool_id}/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/api/tools/{tool_id}/categories/{cid}",
            put(update_category).delete(delete_category),
        )
}
