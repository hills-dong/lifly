CREATE TABLE tool_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_version_id UUID NOT NULL REFERENCES tool_versions(id),
    capability_id UUID NOT NULL REFERENCES atomic_capabilities(id),
    step_order INT NOT NULL,
    input_mapping JSONB DEFAULT '{}',
    output_mapping JSONB DEFAULT '{}',
    condition JSONB,
    on_failure VARCHAR(16) NOT NULL DEFAULT 'abort',
    retry_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tool_version_id, step_order)
);

CREATE INDEX idx_tool_steps_tool_version_id ON tool_steps(tool_version_id);
