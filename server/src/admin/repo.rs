//! Generic, registry-driven CRUD over Postgres.
//!
//! SQL is assembled dynamically, but **all identifiers (table & column names)
//! originate from the static [`registry`](super::registry)** — request input is
//! only ever bound as parameter *values*. Values are bound as text and cast to
//! the column's declared type in SQL (`$1::uuid`, `$1::jsonb`, …), which sidesteps
//! sqlx's need for statically-known bind types while keeping things injection-safe.

use serde_json::Value;
use sqlx::PgPool;

use crate::common::{AppError, AppResult};

use super::registry::{ColType, ResourceSpec};

/// Convert a JSON value into the text form bound for a column of `col_type`.
/// Returns `None` for SQL NULL.
fn json_to_bind(v: &Value, col_type: ColType) -> Option<String> {
    match v {
        Value::Null => None,
        // JSON/JSONB columns: keep the literal JSON text (cast `::jsonb` in SQL).
        _ if col_type == ColType::Json => Some(v.to_string()),
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        // Array/object aimed at a non-JSON column: pass through as text so the
        // database surfaces a clear cast error instead of silently mangling.
        other => Some(other.to_string()),
    }
}

/// Comma-separated list of visible (non-hidden) column names for a SELECT.
fn visible_cols(spec: &ResourceSpec) -> String {
    spec.visible_columns()
        .map(|c| c.name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// List rows with equality filters, sorting and pagination.
///
/// `filters` keys and `sort` MUST already be validated as registry columns by
/// the caller. Returns `(rows, total)`.
pub async fn list(
    pool: &PgPool,
    spec: &ResourceSpec,
    filters: &[(String, String)],
    sort: &str,
    descending: bool,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<Value>, i64)> {
    // WHERE clause from validated filters.
    let mut where_parts = Vec::new();
    for (idx, (col, _)) in filters.iter().enumerate() {
        let cast = spec
            .column(col)
            .expect("filter column validated by caller")
            .col_type
            .pg_cast();
        where_parts.push(format!("{col} = ${}::{cast}", idx + 1));
    }
    let where_sql = if where_parts.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_parts.join(" AND "))
    };

    let dir = if descending { "DESC" } else { "ASC" };
    let order_sql = if sort == spec.pk {
        format!("{sort} {dir}")
    } else {
        format!("{sort} {dir}, {} DESC", spec.pk)
    };

    let limit_idx = filters.len() + 1;
    let offset_idx = filters.len() + 2;
    let list_sql = format!(
        "SELECT row_to_json(t) FROM (SELECT {cols} FROM {table}{where_sql} \
         ORDER BY {order_sql} LIMIT ${limit_idx} OFFSET ${offset_idx}) t",
        cols = visible_cols(spec),
        table = spec.table,
    );

    let mut q = sqlx::query_scalar::<_, Value>(&list_sql);
    for (_, val) in filters {
        q = q.bind(val.clone());
    }
    let rows = q.bind(limit).bind(offset).fetch_all(pool).await?;

    // Total count under the same filters.
    let count_sql = format!("SELECT COUNT(*) FROM {}{where_sql}", spec.table);
    let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
    for (_, val) in filters {
        cq = cq.bind(val.clone());
    }
    let total = cq.fetch_one(pool).await?;

    Ok((rows, total))
}

/// Fetch a single row by primary key, or `None` if absent.
pub async fn get(pool: &PgPool, spec: &ResourceSpec, id: &str) -> AppResult<Option<Value>> {
    let sql = format!(
        "SELECT row_to_json(t) FROM (SELECT {cols} FROM {table} WHERE {pk} = $1::{cast}) t",
        cols = visible_cols(spec),
        table = spec.table,
        pk = spec.pk,
        cast = spec.pk_type.pg_cast(),
    );
    let row = sqlx::query_scalar::<_, Value>(&sql)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// Collect (column, placeholder, bound-value) triples for the writable columns
/// present in `body`, starting placeholders at `$start`.
fn writable_fields<'a>(
    spec: &ResourceSpec,
    body: &'a serde_json::Map<String, Value>,
    start: usize,
) -> Vec<(&'a str, String, Option<String>)> {
    let mut out = Vec::new();
    for (k, v) in body {
        if !spec.is_writable(k) {
            continue; // ignore unknown / read-only fields (e.g. id, created_at)
        }
        let col = spec.column(k).expect("writable implies known");
        let idx = start + out.len();
        out.push((
            k.as_str(),
            format!("${idx}::{}", col.col_type.pg_cast()),
            json_to_bind(v, col.col_type),
        ));
    }
    out
}

/// Insert a new row from `body`; returns the created row (visible columns).
pub async fn create(pool: &PgPool, spec: &ResourceSpec, body: &Value) -> AppResult<Value> {
    let obj = body
        .as_object()
        .ok_or_else(|| AppError::Validation("request body must be a JSON object".into()))?;

    let fields = writable_fields(spec, obj, 1);

    let sql = if fields.is_empty() {
        format!(
            "INSERT INTO {} DEFAULT VALUES RETURNING {}::text",
            spec.table, spec.pk
        )
    } else {
        let cols = fields.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().join(", ");
        let placeholders = fields.iter().map(|(_, p, _)| p.clone()).collect::<Vec<_>>().join(", ");
        format!(
            "INSERT INTO {} ({cols}) VALUES ({placeholders}) RETURNING {}::text",
            spec.table, spec.pk
        )
    };

    let mut q = sqlx::query_scalar::<_, String>(&sql);
    for (_, _, val) in &fields {
        q = q.bind(val.clone());
    }
    let id = q.fetch_one(pool).await?;

    get(pool, spec, &id)
        .await?
        .ok_or_else(|| AppError::Internal("created row vanished".into()))
}

/// Update a row by primary key from `body`; returns the updated row, or `None`
/// if no row matched. A body with no writable fields is a no-op fetch.
pub async fn update(
    pool: &PgPool,
    spec: &ResourceSpec,
    id: &str,
    body: &Value,
) -> AppResult<Option<Value>> {
    let obj = body
        .as_object()
        .ok_or_else(|| AppError::Validation("request body must be a JSON object".into()))?;

    let fields = writable_fields(spec, obj, 1);
    if fields.is_empty() {
        return get(pool, spec, id).await;
    }

    let set_sql = fields
        .iter()
        .map(|(n, p, _)| format!("{n} = {p}"))
        .collect::<Vec<_>>()
        .join(", ");
    let id_idx = fields.len() + 1;
    let sql = format!(
        "UPDATE {} SET {set_sql} WHERE {pk} = ${id_idx}::{cast} RETURNING {pk}::text",
        spec.table,
        pk = spec.pk,
        cast = spec.pk_type.pg_cast(),
    );

    let mut q = sqlx::query_scalar::<_, String>(&sql);
    for (_, _, val) in &fields {
        q = q.bind(val.clone());
    }
    let updated_id = q.bind(id.to_string()).fetch_optional(pool).await?;

    match updated_id {
        Some(uid) => get(pool, spec, &uid).await,
        None => Ok(None),
    }
}

/// Delete a row by primary key. Returns `true` if a row was deleted.
pub async fn delete(pool: &PgPool, spec: &ResourceSpec, id: &str) -> AppResult<bool> {
    let sql = format!(
        "DELETE FROM {} WHERE {pk} = $1::{cast} RETURNING {pk}::text",
        spec.table,
        pk = spec.pk,
        cast = spec.pk_type.pg_cast(),
    );
    let deleted = sqlx::query_scalar::<_, String>(&sql)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;
    Ok(deleted.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_to_bind_handles_types() {
        assert_eq!(json_to_bind(&Value::Null, ColType::Text), None);
        assert_eq!(json_to_bind(&json!("hi"), ColType::Text), Some("hi".into()));
        assert_eq!(json_to_bind(&json!(true), ColType::Bool), Some("true".into()));
        assert_eq!(json_to_bind(&json!(42), ColType::Int), Some("42".into()));
        // JSON column keeps literal JSON text, even for a string value.
        assert_eq!(
            json_to_bind(&json!("hi"), ColType::Json),
            Some("\"hi\"".into())
        );
        assert_eq!(
            json_to_bind(&json!({"a":1}), ColType::Json),
            Some("{\"a\":1}".into())
        );
    }
}
