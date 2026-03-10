-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE data_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_id UUID NOT NULL REFERENCES tools(id),
    pipeline_id UUID,
    parent_id UUID REFERENCES data_objects(id),
    category_id UUID REFERENCES categories(id),
    attributes JSONB NOT NULL DEFAULT '{}',
    vector_embedding vector(1536),
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_data_objects_tool_id ON data_objects(tool_id);
CREATE INDEX idx_data_objects_parent_id ON data_objects(parent_id);
CREATE INDEX idx_data_objects_category_id ON data_objects(category_id);
CREATE INDEX idx_data_objects_status ON data_objects(status);
CREATE INDEX idx_data_objects_attributes ON data_objects USING GIN (attributes);
CREATE INDEX idx_data_objects_vector_embedding ON data_objects USING hnsw (vector_embedding vector_cosine_ops);
