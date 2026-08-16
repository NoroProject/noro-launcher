-- Аккаунт без Discord: операторский вход, который не зависит от чужого сервиса.
--
-- Раньше первого админа заводили руками в SQL, а `discord_id NOT NULL` делал
-- аккаунт без Discord невозможным в принципе.
ALTER TABLE users ALTER COLUMN discord_id DROP NOT NULL;
ALTER TABLE users ALTER COLUMN discord_username DROP NOT NULL;

ALTER TABLE users ADD COLUMN IF NOT EXISTS is_local_account BOOLEAN NOT NULL DEFAULT FALSE;

-- Аккаунт обязан быть хоть чей-то: либо привязан к Discord, либо заведён
-- локально. Без этого пустая запись выглядела бы как валидный пользователь.
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_identity_check;
ALTER TABLE users ADD CONSTRAINT users_identity_check
    CHECK (discord_id IS NOT NULL OR is_local_account);

-- Операторский аккаунт по умолчанию не ходит в игру: ему это незачем, а
-- лишний игровой профиль — лишняя поверхность.
ALTER TABLE users ADD COLUMN IF NOT EXISTS can_play BOOLEAN NOT NULL DEFAULT TRUE;

-- Root нельзя забанить и нельзя удалить: инстанс, оставшийся без операторского
-- входа, чинится только руками в БД.
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_root BOOLEAN NOT NULL DEFAULT FALSE;
CREATE UNIQUE INDEX IF NOT EXISTS users_single_root_idx ON users (is_root) WHERE is_root;

-- Recovery-коды. Не «аварийный вариант», а полноценный вход: WebAuthn не
-- работает по http:// на не-localhost адресе, то есть на типовом первом
-- запуске passkey привязать нельзя вообще, и до настройки домена с TLS это
-- единственный путь внутрь.
CREATE TABLE IF NOT EXISTS recovery_codes (
    id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- argon2: коды короткие и читаются человеком, так что быстрый хеш здесь
    -- перебирается.
    code_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    used_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS recovery_codes_user_idx ON recovery_codes (user_id)
    WHERE used_at IS NULL;
