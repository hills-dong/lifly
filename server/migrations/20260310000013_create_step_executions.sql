CREATE TABLE step_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_id UUID NOT NULL REFERENCES pipelines(id),
    tool_step_id UUID NOT NULL REFERENCES tool_steps(id),
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    actual_input JSONB,
    actual_output JSONB,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms INT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_step_executions_pipeline_id ON step_executions(pipeline_id);
CREATE INDEX idx_step_executions_status ON step_executions(status);
