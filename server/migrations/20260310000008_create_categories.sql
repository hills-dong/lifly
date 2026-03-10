CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_id UUID NOT NULL REFERENCES tools(id),
    parent_id UUID REFERENCES categories(id),
    name VARCHAR(128) NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_categories_tool_id ON categories(tool_id);
CREATE INDEX idx_categories_parent_id ON categories(parent_id);

-- Unique constraint on (tool_id, parent_id, name) handling NULL parent_id
CREATE UNIQUE INDEX uq_categories_tool_parent_name
    ON categories (tool_id, COALESCE(parent_id, '00000000-0000-0000-0000-000000000000'::uuid), name);
