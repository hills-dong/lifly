//! Static resource registry that drives the generic admin CRUD layer.
//!
//! Every manageable table is declared once here. Table and column names used to
//! build SQL come EXCLUSIVELY from this registry — never from request input — so
//! the dynamic SQL in [`super::repo`] cannot be injected through identifiers.
//! User input only ever flows in as bound parameter *values*.

use serde::Serialize;

/// Logical type of a column, used to pick the Postgres cast applied to
/// text-bound parameters (`$1::uuid`, `$1::jsonb`, …) and to inform the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColType {
    Uuid,
    Text,
    Int,
    BigInt,
    Bool,
    Json,
    Timestamp,
    Vector,
}

impl ColType {
    /// The Postgres type used in `$n::<cast>` for parameter values of this column.
    pub fn pg_cast(self) -> &'static str {
        match self {
            ColType::Uuid => "uuid",
            ColType::Text => "text",
            ColType::Int => "int",
            ColType::BigInt => "bigint",
            ColType::Bool => "boolean",
            ColType::Json => "jsonb",
            ColType::Timestamp => "timestamptz",
            ColType::Vector => "vector",
        }
    }
}

/// Declaration of a single column.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ColumnSpec {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub col_type: ColType,
    /// Server-managed; never accepted from create/update payloads (id, timestamps).
    pub readonly: bool,
    /// Excluded from query output (secrets, oversized vectors).
    pub hidden: bool,
}

/// Declaration of a manageable table.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ResourceSpec {
    /// Resource name in the URL; equals the table name.
    pub name: &'static str,
    #[serde(skip)]
    pub table: &'static str,
    pub pk: &'static str,
    #[serde(skip)]
    pub pk_type: ColType,
    pub columns: &'static [ColumnSpec],
}

impl ResourceSpec {
    /// Find a column by name.
    pub fn column(&self, name: &str) -> Option<&ColumnSpec> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Columns visible in query output (non-hidden), in declaration order.
    pub fn visible_columns(&self) -> impl Iterator<Item = &ColumnSpec> {
        self.columns.iter().filter(|c| !c.hidden)
    }

    /// Whether `name` is a writable column (exists and not read-only).
    pub fn is_writable(&self, name: &str) -> bool {
        self.column(name).map(|c| !c.readonly).unwrap_or(false)
    }
}

// ── Column constructors (keep the table declarations terse) ───────────────────

const fn c(name: &'static str, col_type: ColType) -> ColumnSpec {
    ColumnSpec { name, col_type, readonly: false, hidden: false }
}
/// Read-only column (server-managed): id, created_at, updated_at, …
const fn ro(name: &'static str, col_type: ColType) -> ColumnSpec {
    ColumnSpec { name, col_type, readonly: true, hidden: false }
}
/// Hidden + read-only column: secrets / oversized values never exposed or set.
const fn hide(name: &'static str, col_type: ColType) -> ColumnSpec {
    ColumnSpec { name, col_type, readonly: true, hidden: true }
}

use ColType::*;

// ── Per-table column definitions ──────────────────────────────────────────────

static USERS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("username", Text),
    hide("password_hash", Text),
    c("display_name", Text),
    c("preferences", Json),
    ro("created_at", Timestamp),
    ro("updated_at", Timestamp),
];

static DEVICES: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("user_id", Uuid),
    c("name", Text),
    c("device_type", Text),
    c("platform", Text),
    c("token", Text),
    c("is_active", Bool),
    c("last_seen_at", Timestamp),
    ro("created_at", Timestamp),
    ro("updated_at", Timestamp),
];

static ATOMIC_CAPABILITIES: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("name", Text),
    c("description", Text),
    c("category", Text),
    c("runtime_type", Text),
    c("runtime_config", Json),
    c("is_active", Bool),
    ro("created_at", Timestamp),
    ro("updated_at", Timestamp),
];

static CAPABILITY_PARAMS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("capability_id", Uuid),
    c("name", Text),
    c("direction", Text),
    c("data_type", Text),
    c("is_required", Bool),
    c("default_value", Json),
    c("description", Text),
    ro("created_at", Timestamp),
];

static TOOLS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("user_id", Uuid),
    c("name", Text),
    c("description", Text),
    c("source", Text),
    c("status", Text),
    c("data_schema", Json),
    c("trigger_config", Json),
    c("current_version_id", Uuid),
    ro("created_at", Timestamp),
    ro("updated_at", Timestamp),
];

static TOOL_VERSIONS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("tool_id", Uuid),
    c("version_number", Int),
    c("change_log", Text),
    c("data_schema_snapshot", Json),
    c("creator_type", Text),
    ro("created_at", Timestamp),
];

static TOOL_STEPS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("tool_version_id", Uuid),
    c("capability_id", Uuid),
    c("step_order", Int),
    c("input_mapping", Json),
    c("output_mapping", Json),
    c("condition", Json),
    c("on_failure", Text),
    c("retry_count", Int),
    ro("created_at", Timestamp),
];

static CATEGORIES: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("tool_id", Uuid),
    c("parent_id", Uuid),
    c("name", Text),
    c("sort_order", Int),
    ro("created_at", Timestamp),
    ro("updated_at", Timestamp),
];

static DATA_OBJECTS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("tool_id", Uuid),
    c("pipeline_id", Uuid),
    c("parent_id", Uuid),
    c("category_id", Uuid),
    c("attributes", Json),
    // vector(1536): excluded from output (serialization bloat) and never set here.
    hide("vector_embedding", Vector),
    c("status", Text),
    ro("created_at", Timestamp),
    ro("updated_at", Timestamp),
];

static FILE_STORAGE: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("data_object_id", Uuid),
    c("raw_input_id", Uuid),
    c("file_path", Text),
    c("file_name", Text),
    c("mime_type", Text),
    c("file_size", BigInt),
    c("checksum", Text),
    c("role", Text),
    ro("created_at", Timestamp),
];

static RAW_INPUTS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("user_id", Uuid),
    c("device_id", Uuid),
    c("input_type", Text),
    c("raw_content", Text),
    c("metadata", Json),
    c("processing_status", Text),
    ro("created_at", Timestamp),
    ro("updated_at", Timestamp),
];

static PIPELINES: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("tool_id", Uuid),
    c("tool_version_id", Uuid),
    c("raw_input_id", Uuid),
    c("status", Text),
    c("context", Json),
    c("started_at", Timestamp),
    c("completed_at", Timestamp),
    c("error_message", Text),
    ro("created_at", Timestamp),
];

static STEP_EXECUTIONS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("pipeline_id", Uuid),
    c("tool_step_id", Uuid),
    c("status", Text),
    c("actual_input", Json),
    c("actual_output", Json),
    c("started_at", Timestamp),
    c("completed_at", Timestamp),
    c("duration_ms", Int),
    c("error_message", Text),
    ro("created_at", Timestamp),
];

static REMINDERS: &[ColumnSpec] = &[
    ro("id", Uuid),
    c("user_id", Uuid),
    c("data_object_id", Uuid),
    c("title", Text),
    c("description", Text),
    c("trigger_at", Timestamp),
    c("repeat_rule", Json),
    c("status", Text),
    ro("created_at", Timestamp),
    ro("updated_at", Timestamp),
];

/// Helper to declare a resource whose pk is the UUID `id`.
const fn table(name: &'static str, columns: &'static [ColumnSpec]) -> ResourceSpec {
    ResourceSpec {
        name,
        table: name,
        pk: "id",
        pk_type: ColType::Uuid,
        columns,
    }
}

/// All manageable resources, in display order.
static REGISTRY: &[ResourceSpec] = &[
    table("users", USERS),
    table("devices", DEVICES),
    table("atomic_capabilities", ATOMIC_CAPABILITIES),
    table("capability_params", CAPABILITY_PARAMS),
    table("tools", TOOLS),
    table("tool_versions", TOOL_VERSIONS),
    table("tool_steps", TOOL_STEPS),
    table("categories", CATEGORIES),
    table("data_objects", DATA_OBJECTS),
    table("file_storage", FILE_STORAGE),
    table("raw_inputs", RAW_INPUTS),
    table("pipelines", PIPELINES),
    table("step_executions", STEP_EXECUTIONS),
    table("reminders", REMINDERS),
];

/// Return the full registry (for the `/meta` endpoint).
pub fn all() -> &'static [ResourceSpec] {
    REGISTRY
}

/// Look up a resource by its name, if registered.
pub fn find(name: &str) -> Option<&'static ResourceSpec> {
    REGISTRY.iter().find(|r| r.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_all_14_tables() {
        assert_eq!(all().len(), 14);
    }

    #[test]
    fn every_resource_has_a_pk_column() {
        for r in all() {
            assert!(r.column(r.pk).is_some(), "{} missing pk column", r.name);
        }
    }

    #[test]
    fn sensitive_columns_are_hidden() {
        let users = find("users").unwrap();
        assert!(users.column("password_hash").unwrap().hidden);
        let dobj = find("data_objects").unwrap();
        assert!(dobj.column("vector_embedding").unwrap().hidden);
        // Hidden columns are excluded from visible output.
        assert!(!users.visible_columns().any(|c| c.name == "password_hash"));
        assert!(!dobj.visible_columns().any(|c| c.name == "vector_embedding"));
    }

    #[test]
    fn readonly_columns_are_not_writable() {
        let users = find("users").unwrap();
        assert!(!users.is_writable("id"));
        assert!(!users.is_writable("created_at"));
        assert!(users.is_writable("display_name"));
        // Unknown column is never writable.
        assert!(!users.is_writable("definitely_not_a_column"));
    }

    #[test]
    fn unknown_resource_is_none() {
        assert!(find("robots").is_none());
    }
}
