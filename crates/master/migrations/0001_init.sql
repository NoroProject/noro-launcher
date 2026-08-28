-- Initial schema.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS users (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    discord_id       TEXT UNIQUE NOT NULL,
    discord_username TEXT NOT NULL,
    discord_avatar   TEXT,
    mc_uuid          UUID UNIQUE NOT NULL,
    mc_username      VARCHAR(16) UNIQUE NOT NULL,
    skin_url         TEXT,
    cape_url         TEXT,
    banned           BOOLEAN NOT NULL DEFAULT FALSE,
    ban_reason       TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at    TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS roles (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name         TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    color        TEXT,
    is_default   BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order   INT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id    UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission TEXT NOT NULL,
    PRIMARY KEY (role_id, permission)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id    UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS user_permissions (
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission TEXT NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    PRIMARY KEY (user_id, permission)
);

-- Our own bearer tokens for the launcher and the site, issued on top of Discord.
CREATE TABLE IF NOT EXISTS oauth_sessions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token  UUID UNIQUE NOT NULL DEFAULT gen_random_uuid(),
    refresh_token UUID UNIQUE NOT NULL DEFAULT gen_random_uuid(),
    scope         TEXT NOT NULL DEFAULT 'launcher',
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_oauth_user ON oauth_sessions(user_id);

-- Yggdrasil sessions, used by the game server for join/hasJoined.
CREATE TABLE IF NOT EXISTS mc_sessions (
    user_id      UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    access_token UUID NOT NULL,
    client_token TEXT,
    server_id    TEXT,
    ip           TEXT,
    expires_at   TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_mc_sessions_token ON mc_sessions(access_token);

CREATE TABLE IF NOT EXISTS servers (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name           TEXT NOT NULL,
    description    TEXT NOT NULL DEFAULT '',
    icon_url       TEXT,
    background_url TEXT,
    mc_host        TEXT NOT NULL,
    mc_port        INT NOT NULL DEFAULT 25565,
    modloader      TEXT NOT NULL,
    mc_version     TEXT NOT NULL,
    active         BOOLEAN NOT NULL DEFAULT TRUE,
    limited        BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order     INT NOT NULL DEFAULT 0,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS builds (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id          UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    version            TEXT NOT NULL,
    changelog          TEXT NOT NULL DEFAULT '',
    modloader          TEXT NOT NULL,
    modloader_version  TEXT,
    mc_version         TEXT NOT NULL,
    main_class         TEXT NOT NULL DEFAULT '',
    jvm_args           JSONB NOT NULL DEFAULT '[]',
    game_args          JSONB NOT NULL DEFAULT '[]',
    assets_index_name  TEXT NOT NULL DEFAULT '',
    published          BOOLEAN NOT NULL DEFAULT FALSE,
    optional_mods      JSONB NOT NULL DEFAULT '[]',
    unmanaged_paths    JSONB NOT NULL DEFAULT '["saves/","screenshots/","options.txt","logs/","crash-reports/"]',
    user_managed_paths JSONB NOT NULL DEFAULT '[]',
    manifest_signature BYTEA,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_builds_server ON builds(server_id);

-- Mods and configs only. Vanilla artifacts live in mojang_artifacts.
CREATE TABLE IF NOT EXISTS build_files (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    build_id     UUID NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    path         TEXT NOT NULL,
    sha1         TEXT NOT NULL,
    size         BIGINT NOT NULL,
    side         TEXT NOT NULL DEFAULT 'both',
    kind         TEXT NOT NULL DEFAULT 'mod',
    UNIQUE (build_id, path)
);
CREATE INDEX IF NOT EXISTS idx_build_files_build ON build_files(build_id);

-- Mojang and modloader artifacts, shared across builds.
CREATE TABLE IF NOT EXISTS mojang_artifacts (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artifact_type TEXT NOT NULL,
    mc_version    TEXT,
    platform      TEXT,
    path          TEXT NOT NULL,
    sha1          TEXT NOT NULL,
    size          BIGINT NOT NULL,
    UNIQUE (path)
);

-- Uploaded server.jar per server.
CREATE TABLE IF NOT EXISTS server_cores (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id   UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    version     TEXT NOT NULL,
    sha256      TEXT NOT NULL,
    file_sha1   TEXT NOT NULL DEFAULT '',
    size        BIGINT NOT NULL DEFAULT 0,
    active      BOOLEAN NOT NULL DEFAULT FALSE,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS news (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title           TEXT NOT NULL,
    body            TEXT NOT NULL,
    preview_img_url TEXT,
    author_id       UUID REFERENCES users(id),
    pinned          BOOLEAN NOT NULL DEFAULT FALSE,
    published_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Non-interactive tokens for CLI and CI.
CREATE TABLE IF NOT EXISTS admin_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    token_hash  TEXT NOT NULL,
    permissions TEXT[] NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

-- Discord OAuth state (CSRF)
CREATE TABLE IF NOT EXISTS oauth_states (
    state      TEXT PRIMARY KEY,
    redirect   TEXT,
    launcher_port INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS launcher_versions (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version    TEXT NOT NULL,
    platform   TEXT NOT NULL,
    sha256     TEXT NOT NULL,
    file_sha1  TEXT NOT NULL DEFAULT '',   -- FileStore key the download is served from
    size       BIGINT NOT NULL DEFAULT 0,
    signature  TEXT NOT NULL,
    is_current BOOLEAN NOT NULL DEFAULT FALSE,
    built_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (version, platform)
);

CREATE TABLE IF NOT EXISTS launcher_build_jobs (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    github_tag TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'pending',
    log        TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS play_sessions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id     UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    started_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    playtime_secs BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_play_user ON play_sessions(user_id);
