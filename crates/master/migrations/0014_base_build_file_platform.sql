-- Java and natives differ per OS, so a base build file has to say which one it
-- is for.
ALTER TABLE base_build_files
    ADD COLUMN IF NOT EXISTS platform TEXT;

-- NULL means every platform needs it, which is how existing rows behave.
CREATE INDEX IF NOT EXISTS base_build_files_platform
    ON base_build_files (base_build_id, platform);
