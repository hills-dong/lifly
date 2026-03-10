-- Fix tool step input/output mappings to use the engine's dot-path syntax
-- instead of JSONPath. Also make LLM steps skippable so the pipeline works
-- without an LLM configured.

-- ============================================================================
-- Todo List steps (version 00000000-0000-0000-0000-000000000301)
-- Step 1: text_input — capture raw text
-- Step 2: todo_parse (LLM) — parse into structured todo (skip on failure)
-- Step 3: data_object_write — persist as DataObject
-- Step 4: reminder_schedule — create reminder if due_date exists
-- ============================================================================

-- Step 1: text_input
UPDATE tool_steps SET
    input_mapping  = '{"text": "raw_input.raw_content"}',
    output_mapping = '{"captured_text": "result"}'
WHERE id = '00000000-0000-0000-0000-000000000401';

-- Step 2: todo_parse (LLM) — skip on failure so pipeline works without LLM
UPDATE tool_steps SET
    input_mapping  = '{"text": "captured_text"}',
    output_mapping = '{"parsed_data": "result"}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000402';

-- Step 3: data_object_write — use parsed_data if available, else captured_text
UPDATE tool_steps SET
    input_mapping  = '{"data": "parsed_data", "fallback_data": "captured_text"}',
    output_mapping = '{"written_object_id": "data_object_id"}'
WHERE id = '00000000-0000-0000-0000-000000000403';

-- Step 4: reminder_schedule — only run if due_date exists in parsed_data
UPDATE tool_steps SET
    input_mapping  = '{"title": "captured_text", "due_date": "parsed_data.due_date", "data_object_id": "written_object_id"}',
    output_mapping = '{"written_reminder_id": "reminder_id"}',
    condition      = '{"field": "parsed_data.due_date", "exists": true}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000404';

-- ============================================================================
-- ID Document steps (version 00000000-0000-0000-0000-000000000302)
-- Step 1: image_upload — capture image data
-- Step 2: ocr_extract (LLM) — extract fields (skip on failure)
-- Step 3: data_object_write — persist as DataObject
-- Step 4: reminder_schedule — create reminder if expiry_date exists
-- ============================================================================

-- Step 1: image_upload
UPDATE tool_steps SET
    input_mapping  = '{"data": "raw_input.raw_content"}',
    output_mapping = '{"image_data": "result"}'
WHERE id = '00000000-0000-0000-0000-000000000405';

-- Step 2: ocr_extract (LLM) — skip on failure
UPDATE tool_steps SET
    input_mapping  = '{"text": "image_data"}',
    output_mapping = '{"parsed_data": "result"}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000406';

-- Step 3: data_object_write
UPDATE tool_steps SET
    input_mapping  = '{"data": "parsed_data", "fallback_data": "image_data"}',
    output_mapping = '{"written_object_id": "data_object_id"}'
WHERE id = '00000000-0000-0000-0000-000000000407';

-- Step 4: reminder_schedule — only if expiry_date exists
UPDATE tool_steps SET
    input_mapping  = '{"title": "parsed_data.full_name", "due_date": "parsed_data.expiry_date", "data_object_id": "written_object_id"}',
    output_mapping = '{"written_reminder_id": "reminder_id"}',
    condition      = '{"field": "parsed_data.expiry_date", "exists": true}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000408';
