CREATE TABLE tools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    name VARCHAR(128) NOT NULL,
    description TEXT,
    source VARCHAR(16) NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'draft',
    data_schema JSONB,
    trigger_config JSONB DEFAULT '{}',
    current_version_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tools_user_id ON tools(user_id);
CREATE INDEX idx_tools_status ON tools(status);
