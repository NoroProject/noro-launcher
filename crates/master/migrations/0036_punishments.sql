-- Наказания вместо булева флага.
--
-- `users.banned` не знал ни срока, ни автора, ни истории: снятый бан
-- неотличим от «никогда не банили», а «за что» жило в чужой памяти.
CREATE TABLE IF NOT EXISTS punishments (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- ban | warn | server_ban
    kind       TEXT NOT NULL,
    reason     TEXT NOT NULL,
    actor_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_label TEXT NOT NULL,
    -- Только для server_ban: ограничение доступа к одному серверу вместо
    -- тотального бана.
    server_id  UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- NULL — навсегда. Иначе снимается само по истечении.
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    revoked_by UUID REFERENCES users(id) ON DELETE SET NULL,
    -- Предупреждение требует подтверждения прочтения перед входом в игру.
    acknowledged_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS punishments_user_idx ON punishments (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS punishments_active_idx ON punishments (user_id)
    WHERE revoked_at IS NULL;

-- Существующие баны переезжают первой записью: снести их значит разбанить всех.
INSERT INTO punishments (user_id, kind, reason, actor_label, created_at)
SELECT id, 'ban', COALESCE(ban_reason, 'перенесено из users.banned'), 'миграция', created_at
FROM users
WHERE banned;

-- Журнал запусков: снапшот на момент старта игры.
--
-- Таблица есть с миграции 0001, но хранила только playtime: расследование
-- задним числом было невозможно — неизвестно, что за сборка и какие моды были
-- включены. Дополняем её, а не заводим вторую: история запусков должна остаться
-- одной.
ALTER TABLE play_sessions ADD COLUMN IF NOT EXISTS ended_at TIMESTAMPTZ;
ALTER TABLE play_sessions ADD COLUMN IF NOT EXISTS build_version TEXT NOT NULL DEFAULT '';
ALTER TABLE play_sessions ADD COLUMN IF NOT EXISTS launcher_version TEXT NOT NULL DEFAULT '';
ALTER TABLE play_sessions ADD COLUMN IF NOT EXISTS enabled_optional JSONB NOT NULL DEFAULT '[]'::jsonb;
-- Сошлась ли сверка на этом запуске.
ALTER TABLE play_sessions ADD COLUMN IF NOT EXISTS integrity_ok BOOLEAN;
-- playtime заполняется при выходе, а не при старте.
ALTER TABLE play_sessions ALTER COLUMN playtime_secs DROP NOT NULL;

-- Хранится бессрочно: строки лёгкие, а смысл ровно в том, чтобы через полгода
-- увидеть, с какой сборкой играл человек.
CREATE INDEX IF NOT EXISTS play_sessions_user_idx ON play_sessions (user_id, started_at DESC);

-- Внутренние заметки на карточке игрока. Видны только админам.
CREATE TABLE IF NOT EXISTS user_notes (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    author_id  UUID REFERENCES users(id) ON DELETE SET NULL,
    author_label TEXT NOT NULL,
    body       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS user_notes_user_idx ON user_notes (user_id, created_at DESC);

-- Снапшот последней сверки: с чем игрок зашёл в игру.
--
-- Флаги пишутся только при расхождении, а чистый запуск не создаёт ни строки —
-- и журнал запусков остался бы без версии сборки ровно у тех, у кого всё в
-- порядке. Одна строка на пару игрок+сервер: нужен последний снимок.
CREATE TABLE IF NOT EXISTS integrity_snapshots (
    user_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id UUID NOT NULL,
    at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    build_version    TEXT NOT NULL DEFAULT '',
    launcher_version TEXT NOT NULL DEFAULT '',
    enabled_optional JSONB NOT NULL DEFAULT '[]'::jsonb,
    ok        BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (user_id, server_id)
);
