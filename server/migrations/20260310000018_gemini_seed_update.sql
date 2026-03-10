-- Update seed data: switch LLM capabilities to Gemini models, add image_process
-- capability, and re-order 证件管理 pipeline steps to include image processing.

-- ============================================================================
-- 1. Update todo_parse capability to use Gemini 3.1 Pro with text mode
-- ============================================================================
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-3.1-pro","system_prompt":"You are a helpful assistant that extracts structured todo items from user input. Extract the title, description, due date, and priority. Return valid JSON matching the tool data schema.","temperature":0.1,"mode":"text"}'
WHERE id = '00000000-0000-0000-0000-000000000103';

-- ============================================================================
-- 2. Update ocr_extract capability to use Gemini 3.1 Pro with vision mode
-- ============================================================================
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-3.1-pro","system_prompt":"You are a helpful assistant that extracts structured data from identity document images. Extract the certificate type (cert_type), certificate number (cert_number), full name (full_name), expiry date (expiry_date in YYYY-MM-DD format), and issuing country (issuing_country). Return ONLY valid JSON matching this schema, no markdown.","temperature":0.1,"mode":"vision"}'
WHERE id = '00000000-0000-0000-0000-000000000104';

-- ============================================================================
-- 3. Add image_process capability (Gemini 3.1 Flash image generation)
-- ============================================================================
INSERT INTO atomic_capabilities (id, name, description, category, runtime_type, runtime_config)
VALUES (
    '00000000-0000-0000-0000-000000000107',
    'image_process',
    'Uses a remote LLM to process and enhance document images',
    'process',
    'remote_llm',
    '{"model":"gemini-3.1-flash-image","system_prompt":"You are a document image processor. Given a photo of an identity document, enhance the contrast, straighten if needed, crop to the document boundaries, and produce a clean, standardized document image. Output the processed image.","temperature":0.5,"mode":"image_generation"}'
)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 4. Capability params for image_process
-- ============================================================================
INSERT INTO capability_params (id, capability_id, name, direction, data_type, is_required, description)
VALUES
    ('00000000-0000-0000-0000-000000001014', '00000000-0000-0000-0000-000000000107', 'image', 'input', 'file', true, 'The document image to process'),
    ('00000000-0000-0000-0000-000000001015', '00000000-0000-0000-0000-000000000107', 'processed_image', 'output', 'file', true, 'The processed and enhanced document image')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 5. Update 证件管理 pipeline steps
--    Before: 1=image_upload, 2=ocr_extract, 3=data_object_write, 4=reminder_schedule
--    After:  1=image_upload, 2=ocr_extract, 3=image_process, 4=data_object_write, 5=reminder_schedule
-- ============================================================================

-- 5a. Move reminder_schedule from step_order 4 → 5
UPDATE tool_steps SET
    step_order     = 5,
    input_mapping  = '{"title": "parsed_data.full_name", "due_date": "parsed_data.expiry_date", "data_object_id": "written_object_id"}',
    output_mapping = '{"written_reminder_id": "reminder_id"}',
    condition      = '{"field": "parsed_data.expiry_date", "exists": true}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000408';

-- 5b. Move data_object_write from step_order 3 → 4, include processed image
UPDATE tool_steps SET
    step_order     = 4,
    input_mapping  = '{"data": "parsed_data", "fallback_data": "image_data", "processed_image": "processed_image"}',
    output_mapping = '{"written_object_id": "data_object_id"}'
WHERE id = '00000000-0000-0000-0000-000000000407';

-- 5c. Insert image_process as step 3
INSERT INTO tool_steps (id, tool_version_id, capability_id, step_order, input_mapping, output_mapping, on_failure)
VALUES (
    '00000000-0000-0000-0000-000000000409',
    '00000000-0000-0000-0000-000000000302',
    '00000000-0000-0000-0000-000000000107',
    3,
    '{"image_base64": "image_data", "mime_type": "raw_input.mime_type"}',
    '{"processed_image": "image"}',
    'skip'
)
ON CONFLICT DO NOTHING;

-- 5d. Update ocr_extract step to pass image_base64 + mime_type for vision mode
UPDATE tool_steps SET
    input_mapping  = '{"image_base64": "image_data", "mime_type": "raw_input.mime_type"}',
    output_mapping = '{"parsed_data": "result"}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000406';

-- 5e. Update todo_parse step input_mapping to use "text" key
UPDATE tool_steps SET
    input_mapping  = '{"text": "captured_text"}',
    output_mapping = '{"parsed_data": "result"}',
    on_failure     = 'skip'
WHERE id = '00000000-0000-0000-0000-000000000402';

-- 5f. Update todo data_object_write input_mapping to use "data" key
UPDATE tool_steps SET
    input_mapping  = '{"data": "parsed_data", "fallback_data": "captured_text"}',
    output_mapping = '{"written_object_id": "data_object_id"}'
WHERE id = '00000000-0000-0000-0000-000000000403';
