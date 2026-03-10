use sqlx::PgPool;
use std::time::Instant;
use uuid::Uuid;

use crate::common::{AppError, AppResult, AppConfig};
use crate::tool::repo;

use super::executor::StepExecutor;

/// Orchestrates the execution of a pipeline by running each step in order.
pub struct PipelineEngine {
    pool: PgPool,
    config: AppConfig,
}

impl PipelineEngine {
    /// Create a new pipeline engine.
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        Self { pool, config }
    }

    /// Execute a pipeline end-to-end.
    ///
    /// 1. Load the pipeline record.
    /// 2. Load the tool version's steps (ordered by `step_order`).
    /// 3. For each step:
    ///    a. Create a `StepExecution` record (status: running).
    ///    b. Resolve input from pipeline context + `input_mapping`.
    ///    c. Execute the step via `StepExecutor`.
    ///    d. Update the `StepExecution` with output.
    ///    e. Merge output into pipeline context via `output_mapping`.
    ///    f. Handle failure based on the `on_failure` policy.
    /// 4. Update pipeline status to completed or failed.
    pub async fn execute(&self, pipeline_id: Uuid) -> AppResult<()> {
        // 1. Load the pipeline.
        let pipeline = repo::find_pipeline_by_id(&self.pool, pipeline_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("pipeline {pipeline_id} not found")))?;

        // Mark pipeline as running.
        repo::update_pipeline_status(&self.pool, pipeline_id, "running", None).await?;

        // Seed the context with the raw input content.
        let mut context = pipeline
            .context
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));

        // Load raw input and inject into context.
        if let Some(raw_input) = repo::find_raw_input_by_id(&self.pool, pipeline.raw_input_id).await? {
            context["raw_input"] = serde_json::json!({
                "id": raw_input.id,
                "input_type": raw_input.input_type,
                "raw_content": raw_input.raw_content,
                "metadata": raw_input.metadata,
            });
        }

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
                match StepExecutor::execute(&capability, step_input.clone(), &self.config).await {
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

        tracing::info!(pipeline_id = %pipeline_id, "pipeline completed successfully");
        Ok(())
    }

    /// Evaluate a condition against the current pipeline context.
    ///
    /// The condition JSONB is expected to have the form:
    /// ```json
    /// { "field": "raw_input.input_type", "equals": "text" }
    /// ```
    fn evaluate_condition(condition: &serde_json::Value, context: &serde_json::Value) -> bool {
        let field = match condition.get("field").and_then(|f| f.as_str()) {
            Some(f) => f,
            None => return true, // No field specified means the condition is always met.
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

    /// Resolve input for a step by extracting values from the pipeline context
    /// according to the input_mapping.
    ///
    /// The `input_mapping` JSONB maps output keys to context paths:
    /// ```json
    /// { "text": "raw_input.raw_content", "type": "raw_input.input_type" }
    /// ```
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
                    // Use the value directly if it's not a string path.
                    resolved.insert(key.clone(), path_value.clone());
                }
            }
        }

        serde_json::Value::Object(resolved)
    }

    /// Navigate a dotted path (e.g. `"raw_input.raw_content"`) through a JSON value.
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
    ///
    /// The `output_mapping` JSONB maps context keys to output paths:
    /// ```json
    /// { "extracted_data": "result", "summary": "result.summary" }
    /// ```
    fn apply_output_mapping(
        context: &mut serde_json::Value,
        output: &serde_json::Value,
        output_mapping: &Option<serde_json::Value>,
    ) {
        let mapping = match output_mapping {
            Some(m) if m.is_object() => m,
            _ => {
                // No mapping — merge the entire output under a "last_output" key.
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
