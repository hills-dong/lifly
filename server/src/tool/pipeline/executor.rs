use serde_json::Value;

use crate::common::{AppConfig, AppError, AppResult};
use crate::tool::models::AtomicCapability;

/// Dispatches step execution to the appropriate runtime.
pub struct StepExecutor;

impl StepExecutor {
    /// Execute a step by dispatching to the capability's runtime.
    pub async fn execute(
        capability: &AtomicCapability,
        input: Value,
        config: &AppConfig,
    ) -> AppResult<Value> {
        match capability.runtime_type.as_str() {
            "builtin" => Self::execute_builtin(&capability.name, input).await,
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
    async fn execute_builtin(name: &str, input: Value) -> AppResult<Value> {
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
            "data_object_write" => {
                // In a full implementation this would call the data module's repo
                // to write a DataObject. For now, acknowledge the write request.
                let object_type = input
                    .get("object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let data = input.get("data").cloned().unwrap_or(Value::Null);

                tracing::info!(
                    object_type = object_type,
                    "data_object_write: would persist data object"
                );

                Ok(serde_json::json!({
                    "status": "written",
                    "object_type": object_type,
                    "data": data,
                }))
            }
            "reminder_schedule" => {
                // Create a reminder if a due_date is present.
                let title = input
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled reminder");
                let due_date = input.get("due_date").cloned();

                if due_date.is_none() || due_date.as_ref().map_or(true, |v| v.is_null()) {
                    return Err(AppError::Validation(
                        "reminder_schedule requires a due_date field".to_string(),
                    ));
                }

                tracing::info!(
                    title = title,
                    due_date = ?due_date,
                    "reminder_schedule: would create reminder"
                );

                Ok(serde_json::json!({
                    "status": "scheduled",
                    "title": title,
                    "due_date": due_date,
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
