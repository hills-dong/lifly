use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{AppConfig, AppError, AppResult};
use crate::data::repo as data_repo;
use crate::intelligence::repo as reminder_repo;
use crate::tool::models::AtomicCapability;

/// Context about the currently executing pipeline, provided to the executor
/// so it can persist data objects and reminders.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub pool: PgPool,
    pub tool_id: Uuid,
    pub pipeline_id: Uuid,
    pub user_id: Uuid,
}

/// Dispatches step execution to the appropriate runtime.
pub struct StepExecutor;

impl StepExecutor {
    /// Execute a step by dispatching to the capability's runtime.
    pub async fn execute(
        capability: &AtomicCapability,
        input: Value,
        config: &AppConfig,
        ctx: &ExecutionContext,
    ) -> AppResult<Value> {
        match capability.runtime_type.as_str() {
            "builtin" => Self::execute_builtin(&capability.name, input, ctx).await,
            "remote_llm" => {
                Self::execute_remote_llm(input, &capability.runtime_config, config).await
            }
            "script" => Self::execute_script(input, &capability.runtime_config).await,
            other => Err(AppError::Internal(format!(
                "unknown runtime type: {other}"
            ))),
        }
    }

    /// Execute a builtin capability.
    async fn execute_builtin(name: &str, input: Value, ctx: &ExecutionContext) -> AppResult<Value> {
        match name {
            "text_input" => {
                // Pass through the raw content.
                let raw_content = input
                    .get("text")
                    .or_else(|| input.get("raw_content"))
                    .cloned()
                    .unwrap_or(Value::Null);
                Ok(serde_json::json!({
                    "result": raw_content,
                }))
            }
            "image_upload" => {
                // Pass through image data.
                let data = input
                    .get("data")
                    .or_else(|| input.get("raw_content"))
                    .cloned()
                    .unwrap_or(Value::Null);
                Ok(serde_json::json!({
                    "result": data,
                }))
            }
            "data_object_write" => {
                // Use "data" if present, otherwise fall back to "fallback_data".
                let raw_data = input
                    .get("data")
                    .filter(|v| !v.is_null())
                    .or_else(|| input.get("fallback_data"))
                    .cloned()
                    .unwrap_or(Value::Null);

                // If the data is a JSON string, try to parse it.
                let attributes = if let Some(s) = raw_data.as_str() {
                    serde_json::from_str(s).unwrap_or_else(|_| {
                        serde_json::json!({ "content": s })
                    })
                } else if raw_data.is_object() {
                    raw_data
                } else {
                    serde_json::json!({ "content": raw_data })
                };

                let obj = data_repo::create_data_object(
                    &ctx.pool,
                    ctx.tool_id,
                    Some(ctx.pipeline_id),
                    None, // parent_id
                    None, // category_id
                    &attributes,
                )
                .await
                .map_err(AppError::Database)?;

                tracing::info!(
                    data_object_id = %obj.id,
                    tool_id = %ctx.tool_id,
                    "data_object_write: persisted data object"
                );

                Ok(serde_json::json!({
                    "status": "written",
                    "data_object_id": obj.id,
                    "data": attributes,
                }))
            }
            "data_object_query" => {
                // Query data objects for this tool.
                let query = crate::data::DataObjectQuery {
                    tool_id: Some(ctx.tool_id),
                    category_id: None,
                    status: Some("active".to_string()),
                    limit: input.get("limit").and_then(|v| v.as_i64()),
                    offset: None,
                };
                let objects = data_repo::list_data_objects(&ctx.pool, &query)
                    .await
                    .map_err(AppError::Database)?;

                let results: Vec<Value> = objects
                    .into_iter()
                    .map(|o| {
                        serde_json::json!({
                            "id": o.id,
                            "attributes": o.attributes,
                            "status": o.status,
                            "created_at": o.created_at,
                        })
                    })
                    .collect();

                Ok(serde_json::json!({
                    "result": results,
                    "count": results.len(),
                }))
            }
            "reminder_schedule" => {
                let title_raw = input
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled reminder");
                // Truncate long titles.
                let title = if title_raw.len() > 128 {
                    &title_raw[..128]
                } else {
                    title_raw
                };
                let due_date_str = input
                    .get("due_date")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        AppError::Validation(
                            "reminder_schedule requires a due_date string field".to_string(),
                        )
                    })?;

                let trigger_at = chrono::DateTime::parse_from_rfc3339(due_date_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .or_else(|_| {
                        // Try parsing as naive date "YYYY-MM-DD"
                        chrono::NaiveDate::parse_from_str(due_date_str, "%Y-%m-%d")
                            .map(|d| {
                                d.and_hms_opt(9, 0, 0)
                                    .unwrap()
                                    .and_utc()
                            })
                    })
                    .map_err(|_| {
                        AppError::Validation(format!(
                            "invalid due_date format: {due_date_str}, expected RFC3339 or YYYY-MM-DD"
                        ))
                    })?;

                let description = input.get("description").and_then(|v| v.as_str());

                // Get the data_object_id if one was created earlier in the pipeline.
                let data_object_id = input
                    .get("data_object_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok());

                let reminder = reminder_repo::create_reminder(
                    &ctx.pool,
                    ctx.user_id,
                    data_object_id,
                    title,
                    description,
                    trigger_at,
                    None, // repeat_rule
                )
                .await
                .map_err(AppError::Database)?;

                tracing::info!(
                    reminder_id = %reminder.id,
                    title = title,
                    trigger_at = %trigger_at,
                    "reminder_schedule: created reminder"
                );

                Ok(serde_json::json!({
                    "status": "scheduled",
                    "reminder_id": reminder.id,
                    "title": title,
                    "due_date": due_date_str,
                }))
            }
            other => Err(AppError::Internal(format!(
                "unknown builtin capability: {other}"
            ))),
        }
    }

    /// Execute a remote LLM capability by sending a request to the configured API.
    async fn execute_remote_llm(
        input: Value,
        runtime_config: &Option<Value>,
        app_config: &AppConfig,
    ) -> AppResult<Value> {
        let config = runtime_config
            .as_ref()
            .ok_or_else(|| AppError::Internal("remote_llm capability missing runtime_config".to_string()))?;

        let model = config
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("gpt-4");

        let system_prompt = config
            .get("system_prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("You are a helpful assistant.");

        // Extract the user message from input.
        let user_message = if let Some(text) = input.get("text").and_then(|v| v.as_str()) {
            text.to_string()
        } else if let Some(content) = input.get("raw_content").and_then(|v| v.as_str()) {
            content.to_string()
        } else {
            serde_json::to_string(&input)
                .unwrap_or_else(|_| "{}".to_string())
        };

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_message },
            ],
            "temperature": config.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7),
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&app_config.llm_api_url)
            .header("Authorization", format!("Bearer {}", app_config.llm_api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("LLM request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unable to read response body".to_string());
            return Err(AppError::ExternalService(format!(
                "LLM API returned {status}: {body}"
            )));
        }

        let response_body: Value = response
            .json()
            .await
            .map_err(|e| AppError::ExternalService(format!("failed to parse LLM response: {e}")))?;

        // Extract the assistant's message from a standard chat completion response.
        let assistant_content = response_body
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .cloned()
            .unwrap_or(Value::Null);

        Ok(serde_json::json!({
            "result": assistant_content,
            "model": model,
            "raw_response": response_body,
        }))
    }

    /// Execute a script-based capability. Not implemented for M1.
    async fn execute_script(
        _input: Value,
        _runtime_config: &Option<Value>,
    ) -> AppResult<Value> {
        Err(AppError::Internal(
            "script runtime is not implemented in M1".to_string(),
        ))
    }
}
