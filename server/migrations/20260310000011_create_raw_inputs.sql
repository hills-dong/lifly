CREATE TABLE raw_inputs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    device_id UUID REFERENCES devices(id),
    input_type VARCHAR(16) NOT NULL,
    raw_content TEXT,
    metadata JSONB DEFAULT '{}',
    processing_status VARCHAR(16) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_raw_inputs_user_id ON raw_inputs(user_id);
CREATE INDEX idx_raw_inputs_device_id ON raw_inputs(device_id);
CREATE INDEX idx_raw_inputs_processing_status ON raw_inputs(processing_status);

-- Add the deferred FK from file_storage.raw_input_id → raw_inputs.id
ALTER TABLE file_storage
    ADD CONSTRAINT fk_file_storage_raw_input_id
    FOREIGN KEY (raw_input_id) REFERENCES raw_inputs(id);
