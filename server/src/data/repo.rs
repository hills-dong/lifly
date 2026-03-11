use sqlx::PgPool;
use uuid::Uuid;

use super::models::{Category, DataObject, DataObjectQuery, FileStorage};

// ── DataObject ─────────────────────────────────────────────────────────────

/// List data objects with optional filters and pagination.
pub async fn list_data_objects(
    pool: &PgPool,
    query: &DataObjectQuery,
) -> Result<Vec<DataObject>, sqlx::Error> {
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);

    sqlx::query_as::<_, DataObject>(
        "SELECT id, tool_id, pipeline_id, parent_id, category_id,
                attributes, status, created_at, updated_at
         FROM data_objects
         WHERE ($1::uuid IS NULL OR tool_id = $1)
           AND ($2::uuid IS NULL OR category_id = $2)
           AND ($3::varchar IS NULL OR status = $3)
         ORDER BY created_at DESC
         LIMIT $4 OFFSET $5",
    )
    .bind(query.tool_id)
    .bind(query.category_id)
    .bind(&query.status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Find a single data object by id.
pub async fn find_data_object_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<DataObject>, sqlx::Error> {
    sqlx::query_as::<_, DataObject>(
        "SELECT id, tool_id, pipeline_id, parent_id, category_id,
                attributes, status, created_at, updated_at
         FROM data_objects
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Insert a new data object.
pub async fn create_data_object(
    pool: &PgPool,
    tool_id: Uuid,
    pipeline_id: Option<Uuid>,
    parent_id: Option<Uuid>,
    category_id: Option<Uuid>,
    attributes: &serde_json::Value,
) -> Result<DataObject, sqlx::Error> {
    sqlx::query_as::<_, DataObject>(
        "INSERT INTO data_objects
             (id, tool_id, pipeline_id, parent_id, category_id, attributes, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'active', NOW(), NOW())
         RETURNING id, tool_id, pipeline_id, parent_id, category_id,
                   attributes, status, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(tool_id)
    .bind(pipeline_id)
    .bind(parent_id)
    .bind(category_id)
    .bind(attributes)
    .fetch_one(pool)
    .await
}

/// Update a data object's mutable fields.
pub async fn update_data_object(
    pool: &PgPool,
    id: Uuid,
    attributes: Option<&serde_json::Value>,
    category_id: Option<Uuid>,
    status: Option<&str>,
) -> Result<DataObject, sqlx::Error> {
    sqlx::query_as::<_, DataObject>(
        "UPDATE data_objects
         SET attributes  = COALESCE($2, attributes),
             category_id = COALESCE($3, category_id),
             status      = COALESCE($4, status),
             updated_at  = NOW()
         WHERE id = $1
         RETURNING id, tool_id, pipeline_id, parent_id, category_id,
                   attributes, status, created_at, updated_at",
    )
    .bind(id)
    .bind(attributes)
    .bind(category_id)
    .bind(status)
    .fetch_one(pool)
    .await
}

/// Soft-delete a data object by setting its status to 'deleted'.
pub async fn soft_delete_data_object(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_objects SET status = 'deleted', updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// ── FileStorage ────────────────────────────────────────────────────────────

/// List all files associated with a data object.
pub async fn list_files_by_data_object(
    pool: &PgPool,
    data_object_id: Uuid,
) -> Result<Vec<FileStorage>, sqlx::Error> {
    sqlx::query_as::<_, FileStorage>(
        "SELECT id, data_object_id, raw_input_id, file_path, file_name,
                mime_type, file_size, checksum, role, created_at
         FROM file_storage
         WHERE data_object_id = $1
         ORDER BY created_at ASC",
    )
    .bind(data_object_id)
    .fetch_all(pool)
    .await
}

/// Find a single file by id.
pub async fn find_file_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<FileStorage>, sqlx::Error> {
    sqlx::query_as::<_, FileStorage>(
        "SELECT id, data_object_id, raw_input_id, file_path, file_name,
                mime_type, file_size, checksum, role, created_at
         FROM file_storage
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Insert a new file storage record.
#[allow(clippy::too_many_arguments)]
pub async fn create_file_storage(
    pool: &PgPool,
    data_object_id: Option<Uuid>,
    raw_input_id: Option<Uuid>,
    file_path: &str,
    file_name: &str,
    mime_type: &str,
    file_size: i64,
    checksum: &str,
    role: &str,
) -> Result<FileStorage, sqlx::Error> {
    sqlx::query_as::<_, FileStorage>(
        "INSERT INTO file_storage
             (id, data_object_id, raw_input_id, file_path, file_name,
              mime_type, file_size, checksum, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
         RETURNING id, data_object_id, raw_input_id, file_path, file_name,
                   mime_type, file_size, checksum, role, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(data_object_id)
    .bind(raw_input_id)
    .bind(file_path)
    .bind(file_name)
    .bind(mime_type)
    .bind(file_size)
    .bind(checksum)
    .bind(role)
    .fetch_one(pool)
    .await
}

/// Link existing file storage records to a data object by updating their data_object_id.
pub async fn update_file_storage_data_object(
    pool: &PgPool,
    file_storage_id: Uuid,
    data_object_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE file_storage SET data_object_id = $2 WHERE id = $1",
    )
    .bind(file_storage_id)
    .bind(data_object_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Category ───────────────────────────────────────────────────────────────

/// List all categories belonging to a tool, ordered for tree rendering.
pub async fn list_categories(
    pool: &PgPool,
    tool_id: Uuid,
) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "SELECT id, tool_id, parent_id, name, sort_order, created_at, updated_at
         FROM categories
         WHERE tool_id = $1
         ORDER BY sort_order ASC, name ASC",
    )
    .bind(tool_id)
    .fetch_all(pool)
    .await
}

/// Create a new category.
pub async fn create_category(
    pool: &PgPool,
    tool_id: Uuid,
    parent_id: Option<Uuid>,
    name: &str,
    sort_order: i32,
) -> Result<Category, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "INSERT INTO categories (id, tool_id, parent_id, name, sort_order, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
         RETURNING id, tool_id, parent_id, name, sort_order, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(tool_id)
    .bind(parent_id)
    .bind(name)
    .bind(sort_order)
    .fetch_one(pool)
    .await
}

/// Update a category's name and/or sort order.
pub async fn update_category(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    sort_order: Option<i32>,
) -> Result<Category, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "UPDATE categories
         SET name       = COALESCE($2, name),
             sort_order = COALESCE($3, sort_order),
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, tool_id, parent_id, name, sort_order, created_at, updated_at",
    )
    .bind(id)
    .bind(name)
    .bind(sort_order)
    .fetch_one(pool)
    .await
}

/// Search data objects by text query (ILIKE on attributes).
pub async fn search_data_objects(
    pool: &PgPool,
    query: &str,
    tool_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<DataObject>, sqlx::Error> {
    let pattern = format!("%{query}%");
    sqlx::query_as::<_, DataObject>(
        "SELECT id, tool_id, pipeline_id, parent_id, category_id,
                attributes, status, created_at, updated_at
         FROM data_objects
         WHERE status != 'deleted'
           AND ($2::uuid IS NULL OR tool_id = $2)
           AND attributes::text ILIKE $1
         ORDER BY updated_at DESC
         LIMIT $3",
    )
    .bind(&pattern)
    .bind(tool_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Delete a category by id.
pub async fn delete_category(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
