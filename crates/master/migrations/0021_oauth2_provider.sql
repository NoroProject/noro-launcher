-- Migration 0021: Full OAuth2 Provider schema

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
