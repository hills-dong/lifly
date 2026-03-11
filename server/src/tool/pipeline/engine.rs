use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::common::{AppConfig, AppError, AppResult, WsEvent};
use crate::tool::repo;

use super::executor::{ExecutionContext, StepExecutor};

/// Orchestrates the execution of a pipeline by running each step in order.
pub struct PipelineEngine {
    pool: PgPool,
    config: AppConfig,
    ws_tx: Arc<broadcast::Sender<WsEvent>>,
}

impl PipelineEngine {
    /// Create a new pipeline engine.
    pub fn new(pool: PgPool, config: AppConfig, ws_tx: Arc<broadcast::Sender<WsEvent>>) -> Self {
        Self { pool, config, ws_tx }
    }

    /// Broadcast a pipeline status event via WebSocket.
    fn broadcast_status(&self, pipeline_id: Uuid, status: &str, error: Option<&str>) {
        let event = WsEvent {
            event_type: "pipeline.status".to_string(),
            payload: serde_json::json!({
                "pipeline_id": pipeline_id,
                "status": status,
                "error": error,
            }),
        };
        // Ignore send errors (no receivers connected).
        let _ = self.ws_tx.send(event);
    }

    /// Execute a pipeline end-to-end.
    pub async fn execute(&self, pipeline_id: Uuid) -> AppResult<()> {
        // 1. Load the pipeline.
        let pipeline = repo::find_pipeline_by_id(&self.pool, pipeline_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("pipeline {pipeline_id} not found")))?;

        // Mark pipeline as running.
        repo::update_pipeline_status(&self.pool, pipeline_id, "running", None).await?;
        self.broadcast_status(pipeline_id, "running", None);

        // Seed the context with the raw input content.
        let mut context = pipeline
            .context
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));

        // Load raw input and inject into context.
        let raw_input = repo::find_raw_input_by_id(&self.pool, pipeline.raw_input_id).await?;
        let user_id = raw_input.as_ref().map(|r| r.user_id).unwrap_or_default();

        if let Some(ref ri) = raw_input {
            context["raw_input"] = serde_json::json!({
                "id": ri.id,
                "input_type": ri.input_type,
                "raw_content": ri.raw_content,
                "metadata": ri.metadata,
            });
        }

        // Build the execution context for the step executor.
        let exec_ctx = ExecutionContext {
            pool: self.pool.clone(),
            tool_id: pipeline.tool_id,
            pipeline_id,
            user_id,
            file_storage_path: self.config.file_storage_path.clone(),
            raw_input_id: Some(pipeline.raw_input_id),
        };

        // 2. Load the ordered steps for this version.
        let steps = repo::list_steps_by_version(&self.pool, pipeline.tool_version_id).await?;

        // 3. Execute each step.
        for step in &steps {
            // Check if there is a condition that should be evaluated.
            if let Some(condition) = &step.condition {
                if !Self::evaluate_condition(condition, &context) {
                    // Create a skipped step execution record.
                    let exec = repo::create_step_execution(&self.pool, pipeline_id, step.id).await?;
                    repo::update_step_execution(
                        &self.pool,
                        exec.id,
                        "skipped",
                        None,
                        None,
                        None,
                        Some("condition not met"),
                    )
                    .await?;
                    continue;
                }
            }

            // 3a. Create step execution record.
            let exec = repo::create_step_execution(&self.pool, pipeline_id, step.id).await?;

            // 3b. Resolve input from context using input_mapping.
            let step_input = Self::resolve_input(&context, &step.input_mapping);

            // Mark as running with actual input.
            repo::update_step_execution(
                &self.pool,
                exec.id,
                "running",
                Some(&step_input),
                None,
                None,
                None,
            )
            .await?;

            // 3c. Execute the step.
            let start = Instant::now();
            let capability = repo::find_capability_by_id(&self.pool, step.capability_id)
                .await?
                .ok_or_else(|| {
                    AppError::Internal(format!("capability {} not found", step.capability_id))
                })?;

            let mut last_error: Option<String> = None;
            let mut result: Option<serde_json::Value> = None;
            let max_attempts = (step.retry_count + 1) as usize;

            for attempt in 0..max_attempts {
                match StepExecutor::execute(&capability, step_input.clone(), &self.config, &exec_ctx).await {
                    Ok(output) => {
                        result = Some(output);
                        last_error = None;
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e.to_string());
                        if attempt + 1 < max_attempts {
                            tracing::warn!(
                                step_id = %step.id,
                                attempt = attempt + 1,
                                error = %e,
                                "step failed, retrying"
                            );
                        }
                    }
                }
            }

            let duration_ms = start.elapsed().as_millis() as i32;

            if let Some(output) = result {
                // 3d. Update step execution with output.
                repo::update_step_execution(
                    &self.pool,
                    exec.id,
                    "completed",
                    Some(&step_input),
                    Some(&output),
                    Some(duration_ms),
                    None,
                )
                .await?;

                // 3e. Merge output into context using output_mapping.
                Self::apply_output_mapping(&mut context, &output, &step.output_mapping);
                repo::update_pipeline_context(&self.pool, pipeline_id, &context).await?;
            } else {
                // Step failed after all retries.
                let error_msg = last_error.unwrap_or_else(|| "unknown error".to_string());

                repo::update_step_execution(
                    &self.pool,
                    exec.id,
                    "failed",
                    Some(&step_input),
                    None,
                    Some(duration_ms),
                    Some(&error_msg),
                )
                .await?;

                // 3f. Handle failure based on on_failure policy.
                match step.on_failure.as_str() {
                    "skip" => {
                        tracing::warn!(
                            step_id = %step.id,
                            "step failed with skip policy, continuing"
                        );
                        continue;
                    }
                    "abort" | _ => {
                        // Abort the entire pipeline.
                        repo::update_pipeline_status(
                            &self.pool,
                            pipeline_id,
                            "failed",
                            Some(&error_msg),
                        )
                        .await?;
                        repo::update_raw_input_status(
                            &self.pool,
                            pipeline.raw_input_id,
                            "failed",
                        )
                        .await?;
                        self.broadcast_status(pipeline_id, "failed", Some(&error_msg));
                        return Err(AppError::Internal(format!(
                            "pipeline failed at step {}: {error_msg}",
                            step.id
                        )));
                    }
                }
            }
        }

        // 4. Pipeline completed successfully.
        repo::update_pipeline_status(&self.pool, pipeline_id, "completed", None).await?;
        repo::update_raw_input_status(&self.pool, pipeline.raw_input_id, "completed").await?;
        self.broadcast_status(pipeline_id, "completed", None);

        tracing::info!(pipeline_id = %pipeline_id, "pipeline completed successfully");
        Ok(())
    }

    /// Evaluate a condition against the current pipeline context.
    fn evaluate_condition(condition: &serde_json::Value, context: &serde_json::Value) -> bool {
        let field = match condition.get("field").and_then(|f| f.as_str()) {
            Some(f) => f,
            None => return true,
        };

        let actual = Self::resolve_path(context, field);

        if let Some(expected) = condition.get("equals") {
            return actual == *expected;
        }
        if let Some(expected) = condition.get("not_equals") {
            return actual != *expected;
        }
        if condition.get("exists").and_then(|v| v.as_bool()).unwrap_or(false) {
            return !actual.is_null();
        }

        true
    }

    /// Resolve input for a step by extracting values from the pipeline context.
    fn resolve_input(
        context: &serde_json::Value,
        input_mapping: &Option<serde_json::Value>,
    ) -> serde_json::Value {
        let mapping = match input_mapping {
            Some(m) if m.is_object() => m,
            _ => return context.clone(),
        };

        let mut resolved = serde_json::Map::new();
        if let Some(obj) = mapping.as_object() {
            for (key, path_value) in obj {
                if let Some(path) = path_value.as_str() {
                    let value = Self::resolve_path(context, path);
                    resolved.insert(key.clone(), value);
                } else {
                    resolved.insert(key.clone(), path_value.clone());
                }
            }
        }

        serde_json::Value::Object(resolved)
    }

    /// Navigate a dotted path through a JSON value.
    fn resolve_path(value: &serde_json::Value, path: &str) -> serde_json::Value {
        let mut current = value;
        for segment in path.split('.') {
            match current.get(segment) {
                Some(v) => current = v,
                None => return serde_json::Value::Null,
            }
        }
        current.clone()
    }

    /// Apply the output_mapping to merge step output into the pipeline context.
    fn apply_output_mapping(
        context: &mut serde_json::Value,
        output: &serde_json::Value,
        output_mapping: &Option<serde_json::Value>,
    ) {
        let mapping = match output_mapping {
            Some(m) if m.is_object() => m,
            _ => {
                if let Some(ctx) = context.as_object_mut() {
                    ctx.insert("last_output".to_string(), output.clone());
                }
                return;
            }
        };

        if let Some(obj) = mapping.as_object() {
            for (context_key, source_path) in obj {
                let value = if let Some(path) = source_path.as_str() {
                    Self::resolve_path(output, path)
                } else {
                    source_path.clone()
                };

                if let Some(ctx) = context.as_object_mut() {
                    ctx.insert(context_key.clone(), value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── resolve_path tests ─────────────────────────────────────────────

    #[test]
    fn resolve_path_simple_key() {
        let ctx = json!({"name": "Alice"});
        let result = PipelineEngine::resolve_path(&ctx, "name");
        assert_eq!(result, json!("Alice"));
    }

    #[test]
    fn resolve_path_nested_dot_path() {
        let ctx = json!({"a": {"b": {"c": 42}}});
        let result = PipelineEngine::resolve_path(&ctx, "a.b.c");
        assert_eq!(result, json!(42));
    }

    #[test]
    fn resolve_path_missing_key_returns_null() {
        let ctx = json!({"name": "Alice"});
        let result = PipelineEngine::resolve_path(&ctx, "nonexistent");
        assert!(result.is_null());
    }

    #[test]
    fn resolve_path_missing_nested_key_returns_null() {
        let ctx = json!({"a": {"b": 1}});
        let result = PipelineEngine::resolve_path(&ctx, "a.x.y");
        assert!(result.is_null());
    }

    #[test]
    fn resolve_path_empty_segments() {
        // An empty path string split by '.' yields one empty segment "".
        // No key "" exists in a typical object, so it returns Null.
        let ctx = json!({"key": "value"});
        let result = PipelineEngine::resolve_path(&ctx, "");
        assert!(result.is_null());
    }

    #[test]
    fn resolve_path_returns_nested_object() {
        let ctx = json!({"raw_input": {"id": "abc", "content": "hello"}});
        let result = PipelineEngine::resolve_path(&ctx, "raw_input");
        assert!(result.is_object());
        assert_eq!(result["id"], json!("abc"));
    }

    // ── evaluate_condition tests ───────────────────────────────────────

    #[test]
    fn evaluate_condition_null_condition_returns_true() {
        // A condition with no "field" key should always pass.
        let condition = json!({});
        let context = json!({"status": "ok"});
        assert!(PipelineEngine::evaluate_condition(&condition, &context));
    }

    #[test]
    fn evaluate_condition_equals_match() {
        let condition = json!({"field": "status", "equals": "active"});
        let context = json!({"status": "active"});
        assert!(PipelineEngine::evaluate_condition(&condition, &context));
    }

    #[test]
    fn evaluate_condition_equals_no_match() {
        let condition = json!({"field": "status", "equals": "active"});
        let context = json!({"status": "inactive"});
        assert!(!PipelineEngine::evaluate_condition(&condition, &context));
    }

    #[test]
    fn evaluate_condition_not_equals() {
        let condition = json!({"field": "status", "not_equals": "failed"});
        let context = json!({"status": "active"});
        assert!(PipelineEngine::evaluate_condition(&condition, &context));
    }

    #[test]
    fn evaluate_condition_not_equals_when_equal() {
        let condition = json!({"field": "status", "not_equals": "failed"});
        let context = json!({"status": "failed"});
        assert!(!PipelineEngine::evaluate_condition(&condition, &context));
    }

    #[test]
    fn evaluate_condition_exists_true() {
        let condition = json!({"field": "data", "exists": true});
        let context = json!({"data": "something"});
        assert!(PipelineEngine::evaluate_condition(&condition, &context));
    }

    #[test]
    fn evaluate_condition_exists_false_when_missing() {
        let condition = json!({"field": "data", "exists": true});
        let context = json!({"other": "value"});
        assert!(!PipelineEngine::evaluate_condition(&condition, &context));
    }

    #[test]
    fn evaluate_condition_nested_field() {
        let condition = json!({"field": "raw_input.input_type", "equals": "text"});
        let context = json!({"raw_input": {"input_type": "text"}});
        assert!(PipelineEngine::evaluate_condition(&condition, &context));
    }

    #[test]
    fn evaluate_condition_no_operator_returns_true() {
        // Has a field but no equals/not_equals/exists → defaults to true.
        let condition = json!({"field": "status"});
        let context = json!({"status": "active"});
        assert!(PipelineEngine::evaluate_condition(&condition, &context));
    }

    // ── resolve_input tests ────────────────────────────────────────────

    #[test]
    fn resolve_input_no_mapping_returns_full_context() {
        let context = json!({"a": 1, "b": 2});
        let result = PipelineEngine::resolve_input(&context, &None);
        assert_eq!(result, context);
    }

    #[test]
    fn resolve_input_with_mapping() {
        let context = json!({"raw_input": {"raw_content": "hello"}, "step1": {"result": "world"}});
        let mapping = json!({"text": "raw_input.raw_content", "prev": "step1.result"});
        let result = PipelineEngine::resolve_input(&context, &Some(mapping));
        assert_eq!(result["text"], json!("hello"));
        assert_eq!(result["prev"], json!("world"));
    }

    // ── apply_output_mapping tests ─────────────────────────────────────

    #[test]
    fn apply_output_mapping_no_mapping_sets_last_output() {
        let mut context = json!({"existing": true});
        let output = json!({"result": "done"});
        PipelineEngine::apply_output_mapping(&mut context, &output, &None);
        assert_eq!(context["last_output"], output);
    }

    #[test]
    fn apply_output_mapping_with_mapping() {
        let mut context = json!({});
        let output = json!({"result": {"title": "Buy milk"}, "model": "gemini"});
        let mapping = json!({"parsed_data": "result", "llm_model": "model"});
        PipelineEngine::apply_output_mapping(&mut context, &output, &Some(mapping));
        assert_eq!(context["parsed_data"], json!({"title": "Buy milk"}));
        assert_eq!(context["llm_model"], json!("gemini"));
    }
}
