-- Настройки инстанса в БД вместо переменных окружения.
--
-- Что переезжает: публичные URL, CORS, DISCORD_CLIENT_ID, имя инстанса, флаги.
-- Что остаётся в env навсегда: DATABASE_URL, NORO_BIND, NORO_DATA_DIR,
-- SENTRY_DSN (нужен до подъёма БД) и все секреты. Секрет в БД — это секрет в
-- дампе, в бэкапе и в реплике; выигрыша от переезда нет, риск есть.
--
-- Приоритет чтения: env > БД > дефолт. Переменная, заданная явно, побеждает —
-- иначе боевой инстанс, поднятый через compose, перестал бы слушаться своего
-- же окружения.
CREATE TABLE IF NOT EXISTS instance_settings (
    key        TEXT PRIMARY KEY,
    value      JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL
);

-- Состояние первичной настройки. Ровно одна строка: CHECK (id) не даёт
-- завести вторую, а без него «настроен ли инстанс» стало бы вопросом с
-- несколькими ответами.
CREATE TABLE IF NOT EXISTS instance_state (
    id               BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    setup_completed  BOOLEAN NOT NULL DEFAULT FALSE,
    -- argon2 одноразового токена из $NORO_DATA_DIR/setup-token.txt.
    setup_token_hash TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at     TIMESTAMPTZ
);

INSERT INTO instance_state (id) VALUES (TRUE) ON CONFLICT (id) DO NOTHING;
