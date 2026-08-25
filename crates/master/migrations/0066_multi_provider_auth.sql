-- Вход не только через Discord.
--
-- Привязка к платформе была тремя колонками в `users`, поэтому платформа могла
-- быть ровно одна и ровно эта. Привязки уезжают в свою таблицу, ключи
-- провайдеров — в свою.
--
-- Все способы входа одной таблицей: оператор включает и выключает их в одном
-- месте, а сайт одним запросом узнаёт, какие кнопки показывать. У passkey нет
-- ключей приложения — его колонки остаются пустыми.
--
-- Секрет провайдера лежит в БД, в отличие от остальных секретов инстанса
-- (см. 0031). Причина: провайдера заводит оператор из админки, а секрет в env
-- означал бы правку compose и рестарт мастера на каждую новую платформу.
-- Переменная `<METHOD>_CLIENT_SECRET` по-прежнему перекрывает значение из БД,
-- так что боевой инстанс может не класть секрет сюда вовсе.
CREATE TABLE IF NOT EXISTS auth_methods (
    method        TEXT PRIMARY KEY,
    client_id     TEXT NOT NULL DEFAULT '',
    client_secret TEXT NOT NULL DEFAULT '',
    enabled       BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Discord уже настроен: client_id лежал в instance_settings как JSONB-строка,
-- секрет — в env. Переносим id, чтобы после обновления вход не отвалился.
INSERT INTO auth_methods (method, client_id)
SELECT 'discord', COALESCE(value #>> '{}', '')
  FROM instance_settings WHERE key = 'discord_client_id'
ON CONFLICT (method) DO NOTHING;

-- Остальные заводятся выключенными: ключей у них ещё нет, а кнопка входа,
-- которая обрывается на «не настроено», хуже отсутствующей.
INSERT INTO auth_methods (method, enabled) VALUES
    ('discord', TRUE), ('twitch', FALSE), ('google', FALSE), ('passkey', TRUE)
ON CONFLICT (method) DO NOTHING;

-- Привязки. Один аккаунт — сколько угодно платформ, но чужую привязку
-- перехватить нельзя: пара (провайдер, id) уникальна на весь инстанс.
CREATE TABLE IF NOT EXISTS user_identities (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider         TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    username         TEXT,
    avatar_url       TEXT,
    -- Первичная: из неё выведен mc_uuid, её ник и аватар показываются по
    -- умолчанию. Отвязать её нельзя, пока она первичная, — иначе mc_uuid
    -- игрока перестал бы соответствовать хоть чему-нибудь.
    is_primary       BOOLEAN NOT NULL DEFAULT FALSE,
    linked_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (provider, provider_user_id)
);

CREATE INDEX IF NOT EXISTS user_identities_user_idx ON user_identities (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS user_identities_primary_idx
    ON user_identities (user_id) WHERE is_primary;

-- Существующие игроки: их Discord становится первичной привязкой, mc_uuid при
-- этом не меняется — он и был выведен из discord_id.
INSERT INTO user_identities (user_id, provider, provider_user_id, username, avatar_url, is_primary, linked_at)
SELECT id, 'discord', discord_id, discord_username, discord_avatar, TRUE, created_at
  FROM users WHERE discord_id IS NOT NULL
ON CONFLICT (provider, provider_user_id) DO NOTHING;

-- Колонки больше не нужны: две копии одной привязки разъехались бы при первой
-- же смене ника в Discord.
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_identity_check;
ALTER TABLE users DROP COLUMN IF EXISTS discord_id;
ALTER TABLE users DROP COLUMN IF EXISTS discord_username;
ALTER TABLE users DROP COLUMN IF EXISTS discord_avatar;

-- CHECK на «аккаунт обязан быть хоть чей-то» здесь невозможен: условие теперь
-- про строки соседней таблицы. Его держит код — отвязать последнюю платформу у
-- не-локального аккаунта нельзя.

-- Через какого провайдера идёт этот вход и не привязка ли это к уже
-- существующему аккаунту.
ALTER TABLE oauth_states ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT 'discord';
ALTER TABLE oauth_states ADD COLUMN IF NOT EXISTS link_user_id UUID REFERENCES users(id) ON DELETE CASCADE;
