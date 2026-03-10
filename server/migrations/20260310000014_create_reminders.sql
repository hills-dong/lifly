CREATE TABLE reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    data_object_id UUID REFERENCES data_objects(id),
    title VARCHAR(256) NOT NULL,
    description TEXT,
    trigger_at TIMESTAMPTZ NOT NULL,
    repeat_rule JSONB,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_reminders_user_id ON reminders(user_id);
CREATE INDEX idx_reminders_data_object_id ON reminders(data_object_id);
CREATE INDEX idx_reminders_trigger_status ON reminders(trigger_at, status);
