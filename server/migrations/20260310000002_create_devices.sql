CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    name VARCHAR(128) NOT NULL,
    device_type VARCHAR(16) NOT NULL,
    platform VARCHAR(64),
    token VARCHAR(512) UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_seen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_devices_user_id ON devices(user_id);
