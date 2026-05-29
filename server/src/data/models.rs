use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Database models ────────────────────────────────────────────────────────

/// Row from the `data_objects` table.
///
/// Note: `vector_embedding` is intentionally excluded; queries must use
/// explicit column lists that omit it.
#[derive(Debug, Clone, FromRow)]
pub struct DataObject {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub pipeline_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub attributes: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Row from the `file_storage` table.
#[derive(Debug, Clone, FromRow)]
pub struct FileStorage {
    pub id: Uuid,
    pub data_object_id: Option<Uuid>,
    pub raw_input_id: Option<Uuid>,
    pub file_path: String,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub checksum: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

/// Row from the `categories` table.
#[derive(Debug, Clone, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Response DTOs ──────────────────────────────────────────────────────────

/// Summary representation of a data object returned in list endpoints.
#[derive(Debug, Serialize)]
pub struct DataObjectResponse {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub pipeline_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub attributes: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DataObject> for DataObjectResponse {
    fn from(d: DataObject) -> Self {
        Self {
            id: d.id,
            tool_id: d.tool_id,
            pipeline_id: d.pipeline_id,
            parent_id: d.parent_id,
            category_id: d.category_id,
            attributes: d.attributes,
            status: d.status,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

/// Detail representation of a data object (includes associated files).
#[derive(Debug, Serialize)]
pub struct DataObjectDetailResponse {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub pipeline_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub attributes: serde_json::Value,
    pub status: String,
    pub files: Vec<FileStorageResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for `POST /api/data-objects`.
#[derive(Debug, Deserialize)]
pub struct CreateDataObjectRequest {
    pub tool_id: Uuid,
    pub attributes: serde_json::Value,
    pub category_id: Option<Uuid>,
}

/// Request body for `PUT /api/data-objects/:id`.
#[derive(Debug, Deserialize)]
pub struct UpdateDataObjectRequest {
    pub attributes: Option<serde_json::Value>,
    pub category_id: Option<Uuid>,
    pub status: Option<String>,
}

/// Query parameters for `GET /api/data-objects`.
#[derive(Debug, Deserialize)]
pub struct DataObjectQuery {
    pub tool_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Public representation of a stored file.
#[derive(Debug, Serialize)]
pub struct FileStorageResponse {
    pub id: Uuid,
    pub data_object_id: Option<Uuid>,
    pub raw_input_id: Option<Uuid>,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

impl From<FileStorage> for FileStorageResponse {
    fn from(f: FileStorage) -> Self {
        Self {
            id: f.id,
            data_object_id: f.data_object_id,
            raw_input_id: f.raw_input_id,
            file_name: f.file_name,
            mime_type: f.mime_type,
            file_size: f.file_size,
            role: f.role,
            created_at: f.created_at,
        }
    }
}

/// Public representation of a category.
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            tool_id: c.tool_id,
            parent_id: c.parent_id,
            name: c.name,
            sort_order: c.sort_order,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

/// Request body for `POST /api/tools/:tool_id/categories`.
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub sort_order: Option<i32>,
}

/// Request body for `PUT /api/tools/:tool_id/categories/:cid`.
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
}

/// Response returned after a successful file upload.
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub id: Uuid,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub checksum: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

impl From<FileStorage> for UploadResponse {
    fn from(f: FileStorage) -> Self {
        Self {
            id: f.id,
            file_name: f.file_name,
            mime_type: f.mime_type,
            file_size: f.file_size,
            checksum: f.checksum,
            role: f.role,
            created_at: f.created_at,
        }
    }
}
