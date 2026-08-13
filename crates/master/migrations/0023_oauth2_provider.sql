-- Migration 0023: схема OAuth2-провайдера.
--
-- Изначально файл имел номер 0021 и столкнулся с 0021_user_skin_presets:
-- sqlx применил presets, а на этом файле упал с VersionMismatch и оборвал
-- всю дальнейшую цепочку. Таблицы временно создавались из Rust-кода
-- (ensure_default_launcher_app) — тот обход удалён вместе с переименованием.
-- CREATE TABLE IF NOT EXISTS оставлены: на dev и проде таблицы уже созданы
-- тем обходом, и миграция должна лечь поверх них без ошибки.

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

-- Официальный лаунчер — доверенное приложение: экран согласия ему не нужен.
INSERT INTO oauth_applications (client_id, client_secret_hash, name, description, redirect_uris, is_trusted)
VALUES ('noro_launcher', 'public', 'Noro Launcher', 'Официальный лаунчер Noro Network', '["http://127.0.0.1"]', TRUE)
ON CONFLICT (client_id) DO UPDATE SET is_trusted = TRUE;
