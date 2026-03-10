CREATE TABLE capability_params (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    capability_id UUID NOT NULL REFERENCES atomic_capabilities(id),
    name VARCHAR(128) NOT NULL,
    direction VARCHAR(8) NOT NULL,
    data_type VARCHAR(32) NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT false,
    default_value JSONB,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (capability_id, name, direction)
);

CREATE INDEX idx_capability_params_capability_id ON capability_params(capability_id);
