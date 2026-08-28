-- OAuth2 provider schema.
--
-- IF NOT EXISTS is load-bearing: these tables were created from Rust for a while
-- before this file existed, so on older databases they are already there.

CREATE TABLE IF NOT EXISTS oauth_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(64) NOT NULL UNIQUE,
    client_secret_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    icon_url TEXT,
    description TEXT,
    redirect_uris TEXT NOT NULL,
    is_trusted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_authorized_apps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    app_id UUID NOT NULL REFERENCES oauth_applications(id) ON DELETE CASCADE,
    scopes TEXT NOT NULL DEFAULT 'profile',
    authorized_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, app_id)
);

CREATE TABLE IF NOT EXISTS oauth_codes (
    code UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    app_id UUID NOT NULL REFERENCES oauth_applications(id) ON DELETE CASCADE,
    redirect_uri TEXT NOT NULL,
    scopes TEXT NOT NULL DEFAULT 'profile',
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE
);

-- The official launcher is trusted, so it skips the consent screen. It is also a
-- public client — there is no secret to hash.
INSERT INTO oauth_applications (client_id, client_secret_hash, name, description, redirect_uris, is_trusted)
VALUES ('noro_launcher', 'public', 'Noro Launcher', 'Официальный лаунчер Noro Network', '["http://127.0.0.1"]', TRUE)
ON CONFLICT (client_id) DO UPDATE SET is_trusted = TRUE;
