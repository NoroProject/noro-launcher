-- Мастер хранил только core — то, что качает bootstrapper. Сам установщик,
-- который скачивает игрок, не сохранялся нигде, поэтому раздавать его с сайта
-- было нечем.
ALTER TABLE launcher_versions
    ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'core';

-- Уникальность была по (version, platform); теперь одна и та же версия
-- существует в двух видах, и старое ограничение их бы столкнуло.
ALTER TABLE launcher_versions
    DROP CONSTRAINT IF EXISTS launcher_versions_version_platform_key;

CREATE UNIQUE INDEX IF NOT EXISTS launcher_versions_version_platform_kind
    ON launcher_versions (version, platform, kind);

CREATE INDEX IF NOT EXISTS launcher_versions_kind_current
    ON launcher_versions (kind, platform) WHERE is_current;
