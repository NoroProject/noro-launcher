-- Java и natives различаются по ОС, но файлы сборки платформы не знали: мастер
-- раскладывал рантайм только под себя, и на других системах JVM не стартовала.
ALTER TABLE base_build_files
    ADD COLUMN IF NOT EXISTS platform TEXT;

-- NULL значит «нужен всем» — так ведут себя все уже загруженные файлы.
CREATE INDEX IF NOT EXISTS base_build_files_platform
    ON base_build_files (base_build_id, platform);
