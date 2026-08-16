-- Impersonation: вход админа в аккаунт игрока.
--
-- Токен уходит по уже аутентифицированному каналу лаунчера, а не через браузер,
-- URL или аргументы процесса: последние видны в `ps` любому процессу на машине.
CREATE TABLE IF NOT EXISTS impersonation_grants (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Обязательно: единственное, что потом объяснит, зачем это было.
    reason     TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Одноразовый код живёт 60 секунд: он нужен ровно на то, чтобы лаунчер
    -- успел его забрать.
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    revoked_at  TIMESTAMPTZ,
    -- Подтвердил ли админ в лаунчере. NULL — ещё думает.
    accepted    BOOLEAN
);

CREATE INDEX IF NOT EXISTS impersonation_grants_actor_idx
    ON impersonation_grants (actor_id, created_at DESC);

-- Сессия помнит, кто её на самом деле открыл. Без этого действия админа в
-- истории аккаунта неотличимы от действий игрока — а игроку мы не сообщаем,
-- так что разрешать спор придётся только изнутри.
ALTER TABLE oauth_sessions ADD COLUMN IF NOT EXISTS impersonated_by UUID
    REFERENCES users(id) ON DELETE SET NULL;

-- Шаг step-up: успешная проверка открывает окно, внутри которого повторно
-- подтверждаться не нужно. Иначе десять recovery-кодов сгорели бы за десять
-- входов там, где passkey недоступен.
CREATE TABLE IF NOT EXISTS step_up_windows (
    user_id    UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    -- 'passkey' | 'recovery_code'
    method     TEXT NOT NULL,
    confirmed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);
