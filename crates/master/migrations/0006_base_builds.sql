CREATE TABLE IF NOT EXISTS base_builds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mc_version TEXT NOT NULL,
    modloader TEXT NOT NULL,
    modloader_version TEXT,
    main_class TEXT NOT NULL DEFAULT '',
    jvm_args JSONB NOT NULL DEFAULT '[]',
    game_args JSONB NOT NULL DEFAULT '[]',
    assets_index_name TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(mc_version, modloader, modloader_version)
);

CREATE TABLE IF NOT EXISTS base_build_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    base_build_id UUID NOT NULL REFERENCES base_builds(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    sha1 TEXT NOT NULL,
    size BIGINT NOT NULL,
    side TEXT NOT NULL,
    kind TEXT NOT NULL,
    UNIQUE(base_build_id, path)
);

-- Delete all vanilla files from build_files to clean up the DB
-- Since we are moving them to base_build_files, we can safely delete them here.
-- They will be regenerated into base_build_files on the next rebuild.
DELETE FROM build_files WHERE kind IN ('client_jar', 'library', 'native', 'asset', 'asset_index', 'java', 'runtime');
