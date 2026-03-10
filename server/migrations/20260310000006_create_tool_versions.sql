CREATE TABLE tool_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_id UUID NOT NULL REFERENCES tools(id),
    version_number INT NOT NULL,
    change_log TEXT,
    data_schema_snapshot JSONB,
    creator_type VARCHAR(16) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tool_id, version_number)
);

CREATE INDEX idx_tool_versions_tool_id ON tool_versions(tool_id);

-- Add the deferred FK from tools.current_version_id → tool_versions.id
ALTER TABLE tools
    ADD CONSTRAINT fk_tools_current_version_id
    FOREIGN KEY (current_version_id) REFERENCES tool_versions(id);
