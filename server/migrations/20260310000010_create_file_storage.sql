CREATE TABLE file_storage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data_object_id UUID REFERENCES data_objects(id),
    raw_input_id UUID,
    file_path VARCHAR(512) NOT NULL,
    file_name VARCHAR(256) NOT NULL,
    mime_type VARCHAR(128) NOT NULL,
    file_size BIGINT NOT NULL,
    checksum VARCHAR(128) NOT NULL,
    role VARCHAR(16) NOT NULL DEFAULT 'original',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_file_storage_data_object_id ON file_storage(data_object_id);
