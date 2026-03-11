-- Consolidated seed update: replaces migrations 18-21 (gemini_seed_update,
-- fix_gemini_models, use_gemini_31_preview, file_storage_mappings).
-- Contains the FINAL state of all LLM capabilities, image_process capability,
-- and corrected tool_step mappings.

-- ============================================================================
-- 1. Atomic capabilities: update LLM runtime_config to final models
-- ============================================================================

-- todo_parse → gemini-3.1-pro-preview, text mode
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-3.1-pro-preview","system_prompt":"You parse user todo input into a flat JSON object. Return ONLY valid JSON (no markdown, no arrays, no wrapping). The JSON must have exactly these top-level keys: content (string, the original user text preserved exactly), title (string, short summary), description (string or null), due_date (string YYYY-MM-DD or null), priority (string: high/medium/low or null). IMPORTANT: always include the content field with the original input text unchanged.","temperature":0.1,"mode":"text"}'
WHERE id = '00000000-0000-0000-0000-000000000103';

-- ocr_extract → gemini-3.1-pro-preview, vision mode
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-3.1-pro-preview","system_prompt":"You are a document OCR specialist. Given an identity document image, extract all visible fields. Return ONLY valid JSON (no markdown, no code fences) with exactly these keys: cert_type (string, e.g. 居民身份证), cert_number (string), full_name (string), expiry_date (string, YYYY-MM-DD or original text), issuing_country (string). If a field is not visible, set its value to null.","temperature":0.1,"mode":"vision"}'
WHERE id = '00000000-0000-0000-0000-000000000104';

-- ============================================================================
-- 2. Add image_process capability (gemini-3.1-flash-image-preview)
-- ============================================================================
INSERT INTO atomic_capabilities (id, name, description, category, runtime_type, runtime_config)
VALUES (
    '00000000-0000-0000-0000-000000000107',
    'image_process',
    'Uses a remote LLM to process and enhance document images',
    'process',
    'remote_llm',
    '{"model":"gemini-3.1-flash-image-preview","system_prompt":"You are a document image processor. Given a photo of an identity document, enhance the contrast, straighten if needed, crop to the document boundaries, and produce a clean, standardized document image. Output the processed image.","temperature":0.5,"mode":"image_generation"}'
)
ON CONFLICT (id) DO UPDATE SET
    runtime_config = EXCLUDED.runtime_config;

-- ============================================================================
-- 3. Capability params for image_process
-- ============================================================================
INSERT INTO capability_params (id, capability_id, name, direction, data_type, is_required, description)
VALUES
    ('00000000-0000-0000-0000-000000001014', '00000000-0000-0000-0000-000000000107', 'image', 'input', 'file', true, 'The document image to process'),
    ('00000000-0000-0000-0000-000000001015', '00000000-0000-0000-0000-000000000107', 'processed_image', 'output', 'file', true, 'The processed and enhanced document image')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 4. Tool steps: Todo List pipeline (final mappings)
-- ============================================================================

-- Step 2: todo_parse (LLM)
UPDATE tool_steps SET
    input_mapping  = '{"text": "captured_text"}',
    output_mapping = '{"parsed_data": "result"}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000402';

-- Step 3: data_object_write
UPDATE tool_steps SET
    input_mapping  = '{"data": "parsed_data", "fallback_data": "captured_text"}',
    output_mapping = '{"written_object_id": "data_object_id"}'
WHERE id = '00000000-0000-0000-0000-000000000403';

-- ============================================================================
-- 5. Tool steps: 证件管理 pipeline (final mappings + reordering)
-- ============================================================================

-- Step 1: image_upload (add file_storage_id to output)
UPDATE tool_steps SET
    output_mapping = '{"image_data": "result", "original_file_storage_id": "file_storage_id"}'
WHERE id = '00000000-0000-0000-0000-000000000405';

-- Step 2: ocr_extract (vision mode, corrected mime_type path)
UPDATE tool_steps SET
    input_mapping  = '{"image_base64": "image_data", "mime_type": "raw_input.metadata.mime_type"}',
    output_mapping = '{"parsed_data": "result"}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000406';

-- Step 3: image_process (NEW step, inserted)
INSERT INTO tool_steps (id, tool_version_id, capability_id, step_order, input_mapping, output_mapping, on_failure)
VALUES (
    '00000000-0000-0000-0000-000000000409',
    '00000000-0000-0000-0000-000000000302',
    '00000000-0000-0000-0000-000000000107',
    3,
    '{"image_base64": "image_data", "mime_type": "raw_input.metadata.mime_type"}',
    '{"processed_image": "image"}',
    'skip'
)
ON CONFLICT (id) DO UPDATE SET
    input_mapping  = EXCLUDED.input_mapping,
    output_mapping = EXCLUDED.output_mapping,
    on_failure     = EXCLUDED.on_failure;

-- Step 4: data_object_write (moved from 3→4, includes processed_image + file_storage_id)
UPDATE tool_steps SET
    step_order     = 4,
    input_mapping  = '{"data": "parsed_data", "fallback_data": "image_data", "processed_image": "processed_image", "original_file_storage_id": "original_file_storage_id"}',
    output_mapping = '{"written_object_id": "data_object_id"}'
WHERE id = '00000000-0000-0000-0000-000000000407';

-- Step 5: reminder_schedule (moved from 4→5)
UPDATE tool_steps SET
    step_order     = 5,
    input_mapping  = '{"title": "parsed_data.full_name", "due_date": "parsed_data.expiry_date", "data_object_id": "written_object_id"}',
    output_mapping = '{"written_reminder_id": "reminder_id"}',
    condition      = '{"field": "parsed_data.expiry_date", "exists": true}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000408';
