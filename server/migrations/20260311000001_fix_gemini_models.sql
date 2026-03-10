-- Fix Gemini model names to use actually available models.
-- The previous seed used gemini-3.1-pro and gemini-3.1-flash-image which
-- do not exist in the Gemini API, causing 404 errors on all LLM steps.

-- ============================================================================
-- 1. Fix todo_parse: gemini-3.1-pro → gemini-2.5-flash (fast text model)
-- ============================================================================
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-2.5-flash","system_prompt":"You parse user todo input into a flat JSON object. Return ONLY valid JSON (no markdown, no arrays, no wrapping). The JSON must have exactly these top-level keys: content (string, the original user text preserved exactly), title (string, short summary), description (string or null), due_date (string YYYY-MM-DD or null), priority (string: high/medium/low or null). IMPORTANT: always include the content field with the original input text unchanged.","temperature":0.1,"mode":"text"}'
WHERE id = '00000000-0000-0000-0000-000000000103';

-- ============================================================================
-- 2. Fix ocr_extract: gemini-3.1-pro → gemini-2.5-pro (best vision model)
-- ============================================================================
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-2.5-pro","system_prompt":"You are a document OCR specialist. Given an identity document image, extract all visible fields. Return ONLY valid JSON (no markdown, no code fences) with exactly these keys: cert_type (string, e.g. 居民身份证), cert_number (string), full_name (string), expiry_date (string, YYYY-MM-DD or original text), issuing_country (string). If a field is not visible, set its value to null.","temperature":0.1,"mode":"vision"}'
WHERE id = '00000000-0000-0000-0000-000000000104';

-- ============================================================================
-- 3. Fix image_process: gemini-3.1-flash-image → gemini-2.5-flash-image
-- ============================================================================
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-2.5-flash-image","system_prompt":"You are a document image processor. Given a photo of an identity document, enhance the contrast, straighten if needed, crop to the document boundaries, and produce a clean, standardized document image. Output the processed image.","temperature":0.5,"mode":"image_generation"}'
WHERE id = '00000000-0000-0000-0000-000000000107';

-- ============================================================================
-- 4. Fix ocr_extract step input mapping:
--    - mime_type: raw_input.mime_type → raw_input.metadata.mime_type
-- ============================================================================
UPDATE tool_steps SET
    input_mapping = '{"image_base64": "image_data", "mime_type": "raw_input.metadata.mime_type"}'
WHERE id = '00000000-0000-0000-0000-000000000406';

-- ============================================================================
-- 5. Fix image_process step input mapping:
--    - mime_type: raw_input.mime_type → raw_input.metadata.mime_type
-- ============================================================================
UPDATE tool_steps SET
    input_mapping = '{"image_base64": "image_data", "mime_type": "raw_input.metadata.mime_type"}'
WHERE id = '00000000-0000-0000-0000-000000000409';
