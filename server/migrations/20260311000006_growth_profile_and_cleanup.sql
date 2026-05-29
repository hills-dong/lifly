-- Add a generic per-tool `config` JSON (tool-level settings), and use it to store
-- the 成长记录 child profile (birth date + sex). Measurement records keep only
-- date/height_cm/weight_kg; age (months/label) and percentile are computed at display.

ALTER TABLE tools ADD COLUMN IF NOT EXISTS config JSONB NOT NULL DEFAULT '{}'::jsonb;

-- 成长记录 child profile. Birth date back-calculated from the screenshot ages
-- (editable in-app). Sex unknown from the screenshot; defaults to male.
UPDATE tools
SET config = '{"birth_date":"2020-03-12","sex":"male"}'::jsonb
WHERE id = '00000000-0000-0000-0000-000000000203'
  AND (config IS NULL OR config = '{}'::jsonb);

-- Drop the precomputed age fields from the seeded growth records; the UI derives
-- age from the profile birth date now.
UPDATE data_objects
SET attributes = attributes - 'age_months' - 'age_label'
WHERE tool_id = '00000000-0000-0000-0000-000000000203';
