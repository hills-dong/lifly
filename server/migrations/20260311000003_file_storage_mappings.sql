-- Update pipeline step mappings so that file_storage_id from image_upload
-- flows into the pipeline context and gets passed to data_object_write.

-- Step 1 (image_upload, 000000000405): add file_storage_id to output_mapping.
-- Before: {"image_data": "result"}
-- After:  {"image_data": "result", "original_file_storage_id": "file_storage_id"}
UPDATE tool_steps SET
    output_mapping = '{"image_data": "result", "original_file_storage_id": "file_storage_id"}'
WHERE id = '00000000-0000-0000-0000-000000000405';

-- Step 4 (data_object_write, 000000000407): add original_file_storage_id to input_mapping.
-- Before: {"data": "parsed_data", "fallback_data": "image_data", "processed_image": "processed_image"}
-- After:  {"data": "parsed_data", "fallback_data": "image_data", "processed_image": "processed_image", "original_file_storage_id": "original_file_storage_id"}
UPDATE tool_steps SET
    input_mapping = '{"data": "parsed_data", "fallback_data": "image_data", "processed_image": "processed_image", "original_file_storage_id": "original_file_storage_id"}'
WHERE id = '00000000-0000-0000-0000-000000000407';
