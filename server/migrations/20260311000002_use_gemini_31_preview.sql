-- Upgrade models from gemini-2.5-* to gemini-3.1-*-preview.
-- The original seed intended to use 3.1 models but missed the -preview suffix.
-- Now that the pipeline is verified working, use the intended 3.1 models.

-- 1. todo_parse: gemini-2.5-flash → gemini-3.1-pro-preview
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-3.1-pro-preview","system_prompt":"You parse user todo input into a flat JSON object. Return ONLY valid JSON (no markdown, no arrays, no wrapping). The JSON must have exactly these top-level keys: content (string, the original user text preserved exactly), title (string, short summary), description (string or null), due_date (string YYYY-MM-DD or null), priority (string: high/medium/low or null). IMPORTANT: always include the content field with the original input text unchanged.","temperature":0.1,"mode":"text"}'
WHERE id = '00000000-0000-0000-0000-000000000103';

-- 2. ocr_extract: gemini-2.5-pro → gemini-3.1-pro-preview
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-3.1-pro-preview","system_prompt":"You are a document OCR specialist. Given an identity document image, extract all visible fields. Return ONLY valid JSON (no markdown, no code fences) with exactly these keys: cert_type (string, e.g. 居民身份证), cert_number (string), full_name (string), expiry_date (string, YYYY-MM-DD or original text), issuing_country (string). If a field is not visible, set its value to null.","temperature":0.1,"mode":"vision"}'
WHERE id = '00000000-0000-0000-0000-000000000104';

-- 3. image_process: gemini-2.5-flash-image → gemini-3.1-flash-image-preview
UPDATE atomic_capabilities SET
    runtime_config = '{"model":"gemini-3.1-flash-image-preview","system_prompt":"You are a document image processor. Given a photo of an identity document, enhance the contrast, straighten if needed, crop to the document boundaries, and produce a clean, standardized document image. Output the processed image.","temperature":0.5,"mode":"image_generation"}'
WHERE id = '00000000-0000-0000-0000-000000000107';
