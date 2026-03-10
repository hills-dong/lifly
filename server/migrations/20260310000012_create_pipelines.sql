CREATE TABLE pipelines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_id UUID NOT NULL REFERENCES tools(id),
    tool_version_id UUID NOT NULL REFERENCES tool_versions(id),
    raw_input_id UUID REFERENCES raw_inputs(id),
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    context JSONB DEFAULT '{}',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pipelines_tool_id ON pipelines(tool_id);
CREATE INDEX idx_pipelines_raw_input_id ON pipelines(raw_input_id);
CREATE INDEX idx_pipelines_status ON pipelines(status);

-- Add the deferred FK from data_objects.pipeline_id → pipelines.id
ALTER TABLE data_objects
    ADD CONSTRAINT fk_data_objects_pipeline_id
    FOREIGN KEY (pipeline_id) REFERENCES pipelines(id);
