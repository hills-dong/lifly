CREATE TABLE atomic_capabilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(128) UNIQUE NOT NULL,
    description TEXT,
    category VARCHAR(16) NOT NULL,
    runtime_type VARCHAR(16) NOT NULL,
    runtime_config JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_atomic_capabilities_category ON atomic_capabilities(category);
CREATE INDEX idx_atomic_capabilities_runtime_type ON atomic_capabilities(runtime_type);
