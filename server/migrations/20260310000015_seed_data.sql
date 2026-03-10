-- Seed data for Lifly M1: admin user, atomic capabilities, tools, versions, and steps.
-- All INSERTs use ON CONFLICT DO NOTHING to be idempotent.

-- ============================================================================
-- 1. Default admin user
-- ============================================================================
INSERT INTO users (id, username, password_hash, display_name)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$bm90YXJlYWxzYWx0$JKEE39H0BQMaGyjSVLEGdmKQ+zE8sFlA3UPppJ2Bfos',
    'Admin'
)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 2. Atomic capabilities
-- ============================================================================

-- text_input: collect user text input
INSERT INTO atomic_capabilities (id, name, description, category, runtime_type, runtime_config)
VALUES (
    '00000000-0000-0000-0000-000000000101',
    'text_input',
    'Captures text input from the user',
    'collect',
    'builtin',
    '{}'
)
ON CONFLICT DO NOTHING;

-- image_upload: collect user image upload
INSERT INTO atomic_capabilities (id, name, description, category, runtime_type, runtime_config)
VALUES (
    '00000000-0000-0000-0000-000000000102',
    'image_upload',
    'Captures an image uploaded by the user',
    'collect',
    'builtin',
    '{}'
)
ON CONFLICT DO NOTHING;

-- todo_parse: LLM parses todo item from text
INSERT INTO atomic_capabilities (id, name, description, category, runtime_type, runtime_config)
VALUES (
    '00000000-0000-0000-0000-000000000103',
    'todo_parse',
    'Uses a remote LLM to parse a todo item from free-form text',
    'process',
    'remote_llm',
    '{"model":"claude-sonnet-4-20250514","system_prompt":"You are a helpful assistant that extracts structured todo items from user input. Extract the title, description, due date, and priority. Return valid JSON matching the tool data schema.","temperature":0.1}'
)
ON CONFLICT DO NOTHING;

-- ocr_extract: LLM extracts info from document image
INSERT INTO atomic_capabilities (id, name, description, category, runtime_type, runtime_config)
VALUES (
    '00000000-0000-0000-0000-000000000104',
    'ocr_extract',
    'Uses a remote LLM to extract structured information from an identity document image',
    'process',
    'remote_llm',
    '{"model":"claude-sonnet-4-20250514","system_prompt":"You are a helpful assistant that extracts structured data from identity document images. Extract the certificate type, certificate number, full name, expiry date, and issuing country. Return valid JSON matching the tool data schema.","temperature":0.1}'
)
ON CONFLICT DO NOTHING;

-- data_object_write: writes a DataObject to storage
INSERT INTO atomic_capabilities (id, name, description, category, runtime_type, runtime_config)
VALUES (
    '00000000-0000-0000-0000-000000000105',
    'data_object_write',
    'Persists structured data as a DataObject',
    'store',
    'builtin',
    '{}'
)
ON CONFLICT DO NOTHING;

-- reminder_schedule: creates a Reminder
INSERT INTO atomic_capabilities (id, name, description, category, runtime_type, runtime_config)
VALUES (
    '00000000-0000-0000-0000-000000000106',
    'reminder_schedule',
    'Creates a reminder associated with a data object',
    'use',
    'builtin',
    '{}'
)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 3. Capability params
-- ============================================================================

-- text_input params
INSERT INTO capability_params (id, capability_id, name, direction, data_type, is_required, description)
VALUES
    ('00000000-0000-0000-0000-000000001001', '00000000-0000-0000-0000-000000000101', 'text', 'input', 'string', true, 'The raw text entered by the user'),
    ('00000000-0000-0000-0000-000000001002', '00000000-0000-0000-0000-000000000101', 'text', 'output', 'string', true, 'The captured text, passed through unchanged')
ON CONFLICT DO NOTHING;

-- image_upload params
INSERT INTO capability_params (id, capability_id, name, direction, data_type, is_required, description)
VALUES
    ('00000000-0000-0000-0000-000000001003', '00000000-0000-0000-0000-000000000102', 'image', 'input', 'file', true, 'The image file uploaded by the user'),
    ('00000000-0000-0000-0000-000000001004', '00000000-0000-0000-0000-000000000102', 'image_url', 'output', 'string', true, 'URL or path of the stored image')
ON CONFLICT DO NOTHING;

-- todo_parse params
INSERT INTO capability_params (id, capability_id, name, direction, data_type, is_required, description)
VALUES
    ('00000000-0000-0000-0000-000000001005', '00000000-0000-0000-0000-000000000103', 'text', 'input', 'string', true, 'Free-form text to parse into a todo'),
    ('00000000-0000-0000-0000-000000001006', '00000000-0000-0000-0000-000000000103', 'structured_data', 'output', 'json', true, 'Parsed todo object matching the data schema')
ON CONFLICT DO NOTHING;

-- ocr_extract params
INSERT INTO capability_params (id, capability_id, name, direction, data_type, is_required, description)
VALUES
    ('00000000-0000-0000-0000-000000001007', '00000000-0000-0000-0000-000000000104', 'image_url', 'input', 'string', true, 'URL or path of the document image'),
    ('00000000-0000-0000-0000-000000001008', '00000000-0000-0000-0000-000000000104', 'structured_data', 'output', 'json', true, 'Extracted document fields matching the data schema')
ON CONFLICT DO NOTHING;

-- data_object_write params
INSERT INTO capability_params (id, capability_id, name, direction, data_type, is_required, description)
VALUES
    ('00000000-0000-0000-0000-000000001009', '00000000-0000-0000-0000-000000000105', 'structured_data', 'input', 'json', true, 'The structured data to persist'),
    ('00000000-0000-0000-0000-000000001010', '00000000-0000-0000-0000-000000000105', 'data_object_id', 'output', 'uuid', true, 'ID of the created DataObject')
ON CONFLICT DO NOTHING;

-- reminder_schedule params
INSERT INTO capability_params (id, capability_id, name, direction, data_type, is_required, description)
VALUES
    ('00000000-0000-0000-0000-000000001011', '00000000-0000-0000-0000-000000000106', 'data_object_id', 'input', 'uuid', true, 'The DataObject to associate the reminder with'),
    ('00000000-0000-0000-0000-000000001012', '00000000-0000-0000-0000-000000000106', 'trigger_at', 'input', 'datetime', false, 'When to trigger the reminder (extracted from data if available)'),
    ('00000000-0000-0000-0000-000000001013', '00000000-0000-0000-0000-000000000106', 'reminder_id', 'output', 'uuid', true, 'ID of the created Reminder')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 4. Tools
-- ============================================================================

-- Todo List tool
INSERT INTO tools (id, user_id, name, description, source, status, data_schema)
VALUES (
    '00000000-0000-0000-0000-000000000201',
    '00000000-0000-0000-0000-000000000001',
    'Todo List',
    'Capture and manage todo items from natural language input',
    'system',
    'active',
    '{"type":"object","properties":{"title":{"type":"string"},"description":{"type":"string"},"due_date":{"type":"string","format":"date-time"},"priority":{"type":"string","enum":["low","normal","high","urgent"]},"completed":{"type":"boolean"}}}'
)
ON CONFLICT DO NOTHING;

-- ID Document tool (证件管理)
INSERT INTO tools (id, user_id, name, description, source, status, data_schema)
VALUES (
    '00000000-0000-0000-0000-000000000202',
    '00000000-0000-0000-0000-000000000001',
    '证件管理',
    'Capture and manage identity documents by extracting info from photos',
    'system',
    'active',
    '{"type":"object","properties":{"cert_type":{"type":"string"},"cert_number":{"type":"string"},"full_name":{"type":"string"},"expiry_date":{"type":"string","format":"date"},"issuing_country":{"type":"string"}}}'
)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 5. Tool versions
-- ============================================================================

-- Todo List v1
INSERT INTO tool_versions (id, tool_id, version_number, change_log, data_schema_snapshot, creator_type)
VALUES (
    '00000000-0000-0000-0000-000000000301',
    '00000000-0000-0000-0000-000000000201',
    1,
    'Initial version',
    '{"type":"object","properties":{"title":{"type":"string"},"description":{"type":"string"},"due_date":{"type":"string","format":"date-time"},"priority":{"type":"string","enum":["low","normal","high","urgent"]},"completed":{"type":"boolean"}}}',
    'system'
)
ON CONFLICT DO NOTHING;

-- ID Document v1
INSERT INTO tool_versions (id, tool_id, version_number, change_log, data_schema_snapshot, creator_type)
VALUES (
    '00000000-0000-0000-0000-000000000302',
    '00000000-0000-0000-0000-000000000202',
    1,
    'Initial version',
    '{"type":"object","properties":{"cert_type":{"type":"string"},"cert_number":{"type":"string"},"full_name":{"type":"string"},"expiry_date":{"type":"string","format":"date"},"issuing_country":{"type":"string"}}}',
    'system'
)
ON CONFLICT DO NOTHING;

-- Set current_version_id on tools
UPDATE tools SET current_version_id = '00000000-0000-0000-0000-000000000301'
WHERE id = '00000000-0000-0000-0000-000000000201' AND current_version_id IS NULL;

UPDATE tools SET current_version_id = '00000000-0000-0000-0000-000000000302'
WHERE id = '00000000-0000-0000-0000-000000000202' AND current_version_id IS NULL;

-- ============================================================================
-- 6. Tool steps
-- ============================================================================

-- Todo List steps: text_input → todo_parse → data_object_write → reminder_schedule
INSERT INTO tool_steps (id, tool_version_id, capability_id, step_order, input_mapping, output_mapping, on_failure)
VALUES
    ('00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000101', 1, '{}', '{}', 'abort'),
    ('00000000-0000-0000-0000-000000000402', '00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000103', 2, '{"text":"$.steps[0].output.text"}', '{}', 'abort'),
    ('00000000-0000-0000-0000-000000000403', '00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000105', 3, '{"structured_data":"$.steps[1].output.structured_data"}', '{}', 'abort'),
    ('00000000-0000-0000-0000-000000000404', '00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000106', 4, '{"data_object_id":"$.steps[2].output.data_object_id","trigger_at":"$.steps[1].output.structured_data.due_date"}', '{}', 'abort')
ON CONFLICT DO NOTHING;

-- ID Document steps: image_upload → ocr_extract → data_object_write → reminder_schedule
INSERT INTO tool_steps (id, tool_version_id, capability_id, step_order, input_mapping, output_mapping, on_failure)
VALUES
    ('00000000-0000-0000-0000-000000000405', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000102', 1, '{}', '{}', 'abort'),
    ('00000000-0000-0000-0000-000000000406', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000104', 2, '{"image_url":"$.steps[0].output.image_url"}', '{}', 'abort'),
    ('00000000-0000-0000-0000-000000000407', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000105', 3, '{"structured_data":"$.steps[1].output.structured_data"}', '{}', 'abort'),
    ('00000000-0000-0000-0000-000000000408', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000106', 4, '{"data_object_id":"$.steps[2].output.data_object_id","trigger_at":"$.steps[1].output.structured_data.expiry_date"}', '{}', 'abort')
ON CONFLICT DO NOTHING;
