use sqlx::PgPool;
use uuid::Uuid;

use super::models::{AtomicCapability, CapabilityParam};

/// List all capabilities, optionally filtered by category.
pub async fn list_capabilities(
    pool: &PgPool,
    category: Option<&str>,
) -> Result<Vec<AtomicCapability>, sqlx::Error> {
    match category {
        Some(cat) => {
            sqlx::query_as::<_, AtomicCapability>(
                "SELECT id, name, description, category, runtime_type, runtime_config, \
                        is_active, created_at, updated_at \
                 FROM atomic_capabilities \
                 WHERE category = $1 \
                 ORDER BY created_at",
            )
            .bind(cat)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, AtomicCapability>(
                "SELECT id, name, description, category, runtime_type, runtime_config, \
                        is_active, created_at, updated_at \
                 FROM atomic_capabilities \
                 ORDER BY created_at",
            )
            .fetch_all(pool)
            .await
        }
    }
}

/// Find a single capability by its primary key.
pub async fn find_capability_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<AtomicCapability>, sqlx::Error> {
    sqlx::query_as::<_, AtomicCapability>(
        "SELECT id, name, description, category, runtime_type, runtime_config, \
                is_active, created_at, updated_at \
         FROM atomic_capabilities \
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// List all params that belong to a given capability.
pub async fn list_params_by_capability(
    pool: &PgPool,
    capability_id: Uuid,
) -> Result<Vec<CapabilityParam>, sqlx::Error> {
    sqlx::query_as::<_, CapabilityParam>(
        "SELECT id, capability_id, name, direction, data_type, is_required, \
                default_value, description, created_at \
         FROM capability_params \
         WHERE capability_id = $1 \
         ORDER BY direction, name",
    )
    .bind(capability_id)
    .fetch_all(pool)
    .await
}
