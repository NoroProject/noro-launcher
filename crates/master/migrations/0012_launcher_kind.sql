-- Splits the core the bootstrapper pulls from the installer a player downloads
-- off the site. Only the core used to be stored, so there was nothing to serve.
ALTER TABLE launcher_versions
    ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'core';

-- One version now exists in two kinds, so uniqueness has to include it.
ALTER TABLE launcher_versions
    DROP CONSTRAINT IF EXISTS launcher_versions_version_platform_key;

CREATE UNIQUE INDEX IF NOT EXISTS launcher_versions_version_platform_kind
    ON launcher_versions (version, platform, kind);

CREATE INDEX IF NOT EXISTS launcher_versions_kind_current
    ON launcher_versions (kind, platform) WHERE is_current;
