use std::path::PathBuf;

use base64::Engine;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::common::{AppConfig, AppError, AppResult};
use crate::data::repo as data_repo;
use crate::intelligence::repo as reminder_repo;
use crate::tool::models::AtomicCapability;

use super::gemini;

/// Context about the currently executing pipeline, provided to the executor
/// so it can persist data objects and reminders.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub pool: PgPool,
    pub tool_id: Uuid,
    pub pipeline_id: Uuid,
    pub user_id: Uuid,
    pub file_storage_path: PathBuf,
    pub raw_input_id: Option<Uuid>,
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
                // Extract base64 image data from input.
                let data = input
                    .get("data")
                    .or_else(|| input.get("raw_content"))
                    .cloned()
                    .unwrap_or(Value::Null);

                let base64_str = match data.as_str() {
                    Some(s) => s.to_string(),
                    None => {
                        // No image data — pass through as before.
                        return Ok(serde_json::json!({ "result": data }));
                    }
                };

                // Decode base64 to bytes and persist to disk + DB.
                let file_record = Self::save_base64_to_file(
                    &base64_str,
                    "image/png", // default; metadata may refine later
                    "original",
                    None, // no data_object_id yet
                    ctx.raw_input_id,
                    &ctx.pool,
                    &ctx.file_storage_path,
                )
                .await?;

                tracing::info!(
                    file_storage_id = %file_record.id,
                    file_path = %file_record.file_path,
                    "image_upload: persisted original image to disk"
                );

                // Return base64 data (for downstream LLM steps) plus file_storage_id.
                Ok(serde_json::json!({
                    "result": base64_str,
                    "file_storage_id": file_record.id.to_string(),
                    "file_path": file_record.file_path,
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
                let parsed = if let Some(s) = raw_data.as_str() {
                    serde_json::from_str(s).unwrap_or_else(|_| {
                        serde_json::json!({ "content": s })
                    })
                } else if raw_data.is_object() {
                    raw_data
                } else {
                    serde_json::json!({ "content": raw_data })
                };

                // If parsed data has a single-item array wrapper (e.g. {"todos": [{...}]}),
                // unwrap the first element and merge with the wrapper's other fields.
                let attributes = if let Some(obj) = parsed.as_object() {
                    let arrays: Vec<_> = obj
                        .iter()
                        .filter(|(_, v)| v.is_array())
                        .collect();
                    if arrays.len() == 1 {
                        let (_, arr_val) = arrays[0];
                        if let Some(arr) = arr_val.as_array() {
                            if arr.len() == 1 {
                                if let Some(item) = arr[0].as_object() {
                                    // Merge non-array fields with the item
                                    let mut merged = item.clone();
                                    for (k, v) in obj {
                                        if !v.is_array() {
                                            merged.insert(k.clone(), v.clone());
                                        }
                                    }
                                    Value::Object(merged)
                                } else {
                                    parsed
                                }
                            } else {
                                parsed
                            }
                        } else {
                            parsed
                        }
                    } else {
                        parsed
                    }
                } else {
                    parsed
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

                // Link any original image FileStorage record (from image_upload step)
                // to this data object.
                if let Some(original_fs_id) = input
                    .get("original_file_storage_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                {
                    if let Err(e) = data_repo::update_file_storage_data_object(
                        &ctx.pool,
                        original_fs_id,
                        obj.id,
                    )
                    .await
                    {
                        tracing::warn!(
                            error = %e,
                            "data_object_write: failed to link original FileStorage to DataObject"
                        );
                    }
                }

                // Handle processed image if present: decode, save to disk, create FileStorage record.
                if let Some(processed) = input.get("processed_image") {
                    // The processed_image comes from the image_generation LLM step output
                    // as {"mime_type": "...", "data": "<base64>"}
                    let proc_base64 = processed
                        .get("data")
                        .and_then(|v| v.as_str());
                    let proc_mime = processed
                        .get("mime_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("image/png");

                    if let Some(b64) = proc_base64 {
                        match Self::save_base64_to_file(
                            b64,
                            proc_mime,
                            "processed",
                            Some(obj.id),
                            ctx.raw_input_id,
                            &ctx.pool,
                            &ctx.file_storage_path,
                        )
                        .await
                        {
                            Ok(record) => {
                                tracing::info!(
                                    file_storage_id = %record.id,
                                    data_object_id = %obj.id,
                                    "data_object_write: persisted processed image"
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "data_object_write: failed to persist processed image"
                                );
                            }
                        }
                    }
                }

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

    /// Decode a base64 string, write the bytes to disk under the file storage
    /// directory, and create a `file_storage` DB record.
    ///
    /// Returns the created `FileStorage` record.
    async fn save_base64_to_file(
        base64_data: &str,
        mime_type: &str,
        role: &str,
        data_object_id: Option<Uuid>,
        raw_input_id: Option<Uuid>,
        pool: &PgPool,
        file_storage_path: &std::path::Path,
    ) -> AppResult<crate::data::models::FileStorage> {
        use base64::engine::general_purpose::STANDARD;

        // Decode base64.
        let bytes = STANDARD.decode(base64_data).map_err(|e| {
            AppError::Validation(format!("invalid base64 data: {e}"))
        })?;

        // Determine file extension from MIME type.
        let extension = match mime_type {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/bmp" => "bmp",
            "image/tiff" => "tiff",
            "application/pdf" => "pdf",
            _ => "bin",
        };

        // Build storage path: {storage_path}/{year}/{month}/{uuid}.{ext}
        let now = chrono::Utc::now();
        let year = now.format("%Y");
        let month = now.format("%m");
        let file_uuid = Uuid::new_v4();
        let file_name = format!("{file_uuid}.{extension}");
        let relative_path = format!("{year}/{month}/{file_name}");
        let full_dir = file_storage_path.join(format!("{year}/{month}"));
        let full_path = file_storage_path.join(&relative_path);

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

        // Create database record.
        let record = data_repo::create_file_storage(
            pool,
            data_object_id,
            raw_input_id,
            &relative_path,
            &file_name,
            mime_type,
            file_size,
            &checksum,
            role,
        )
        .await
        .map_err(AppError::Database)?;

        Ok(record)
    }

    /// Execute a remote LLM capability by sending a request to the Gemini API.
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
            .unwrap_or("gemini-3.1-pro");

        let system_prompt = config
            .get("system_prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("You are a helpful assistant.");

        let temperature = config.get("temperature").and_then(|v| v.as_f64());

        let mode = config
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        let request_body = match mode {
            "vision" => {
                let image_base64 = input
                    .get("image_base64")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mime_type = input
                    .get("mime_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("image/png");
                let text = input
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Analyze this image and follow the instructions in your system prompt.");
                gemini::build_image_request(image_base64, mime_type, text, system_prompt, temperature)
            }
            "image_generation" => {
                let image_base64 = input
                    .get("image_base64")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mime_type = input
                    .get("mime_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("image/png");
                let text = input
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Process this image.");
                gemini::build_image_generation_request(image_base64, mime_type, text, system_prompt, temperature)
            }
            _ => {
                // Default "text" mode.
                let user_message = if let Some(text) = input.get("text").and_then(|v| v.as_str()) {
                    text.to_string()
                } else if let Some(content) = input.get("raw_content").and_then(|v| v.as_str()) {
                    content.to_string()
                } else {
                    serde_json::to_string(&input).unwrap_or_else(|_| "{}".to_string())
                };
                gemini::build_text_request(&user_message, system_prompt, temperature)
            }
        };

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            app_config.llm_api_url.trim_end_matches('/'),
            model,
            app_config.llm_api_key,
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
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

        let gemini_resp: gemini::GeminiResponse = serde_json::from_value(response_body.clone())
            .map_err(|e| AppError::ExternalService(format!("failed to parse Gemini response: {e}")))?;

        if mode == "image_generation" {
            let text_result = gemini::extract_text(&gemini_resp).unwrap_or_default();
            let image = gemini::extract_image(&gemini_resp).map(|img| {
                serde_json::json!({
                    "mime_type": img.mime_type,
                    "data": img.data,
                })
            });
            Ok(serde_json::json!({
                "result": text_result,
                "image": image,
                "model": model,
                "raw_response": response_body,
            }))
        } else {
            let text_result = gemini::extract_text(&gemini_resp).unwrap_or_default();
            // Try to parse the text as JSON so downstream steps can access fields.
            let result_value = Self::try_parse_json_response(&text_result);
            Ok(serde_json::json!({
                "result": result_value,
                "model": model,
                "raw_response": response_body,
            }))
        }
    }

    /// Try to parse an LLM text response as JSON.
    /// Handles raw JSON, markdown-fenced JSON (```json ... ```), and plain text.
    fn try_parse_json_response(text: &str) -> Value {
        let trimmed = text.trim();

        // Try direct parse first.
        if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
            if val.is_object() || val.is_array() {
                return val;
            }
        }

        // Try extracting from markdown code fences: ```json ... ``` or ``` ... ```
        if let Some(start) = trimmed.find("```") {
            let after_fence = &trimmed[start + 3..];
            // Skip optional language tag (e.g., "json")
            let content_start = after_fence.find('\n').map(|i| i + 1).unwrap_or(0);
            let content = &after_fence[content_start..];
            if let Some(end) = content.find("```") {
                let json_str = content[..end].trim();
                if let Ok(val) = serde_json::from_str::<Value>(json_str) {
                    if val.is_object() || val.is_array() {
                        return val;
                    }
                }
            }
        }

        // Return as plain string if not parseable.
        Value::String(text.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn try_parse_json_response_valid_object() {
        let input = r#"{"name": "Alice", "age": 30}"#;
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_object());
        assert_eq!(result["name"], json!("Alice"));
        assert_eq!(result["age"], json!(30));
    }

    #[test]
    fn try_parse_json_response_valid_array() {
        let input = r#"[1, 2, 3]"#;
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_array());
        assert_eq!(result, json!([1, 2, 3]));
    }

    #[test]
    fn try_parse_json_response_markdown_fence_with_json_tag() {
        let input = "Here is the result:\n```json\n{\"key\": \"value\"}\n```\nDone.";
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_object());
        assert_eq!(result["key"], json!("value"));
    }

    #[test]
    fn try_parse_json_response_markdown_fence_no_language_tag() {
        let input = "```\n{\"a\": 1}\n```";
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_object());
        assert_eq!(result["a"], json!(1));
    }

    #[test]
    fn try_parse_json_response_plain_text() {
        let input = "This is just plain text, not JSON at all.";
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_string());
        assert_eq!(result.as_str().unwrap(), input);
    }

    #[test]
    fn try_parse_json_response_empty_string() {
        let input = "";
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_string());
        assert_eq!(result.as_str().unwrap(), "");
    }

    #[test]
    fn try_parse_json_response_nested_markdown_fences() {
        // When the JSON value itself contains ```, the parser's inner find("```")
        // matches inside the content, resulting in invalid JSON extraction.
        // The function falls back to returning the original text as a string.
        let input = "```json\n{\"nested\": \"```inner```\"}\n```";
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_string());
        assert_eq!(result.as_str().unwrap(), input);
    }

    #[test]
    fn try_parse_json_response_whitespace_around_json() {
        let input = "  \n  {\"trimmed\": true}  \n  ";
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_object());
        assert_eq!(result["trimmed"], json!(true));
    }

    #[test]
    fn try_parse_json_response_array_in_markdown_fence() {
        let input = "```json\n[{\"id\": 1}, {\"id\": 2}]\n```";
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_array());
        assert_eq!(result[0]["id"], json!(1));
    }

    #[test]
    fn try_parse_json_response_scalar_json_returns_string() {
        // A JSON scalar (number/string) is not object or array, so it falls through.
        let input = "42";
        let result = StepExecutor::try_parse_json_response(input);
        assert!(result.is_string());
        assert_eq!(result.as_str().unwrap(), "42");
    }
}
