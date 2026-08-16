-- Настоящая проверка WebAuthn вместо доверия к credential_id.
--
-- Старый вход находил запись по credential_id и сразу выдавал сессию: подпись
-- не проверялась, public_key и counter не использовались. credential_id —
-- публичное значение, его отдаёт браузер, так что «знаешь id — вошёл».
--
-- Материала для настоящей проверки в старых записях нет (public_key лежит в
-- произвольном формате, привязки к challenge не было), а оставить их — значит
-- оставить обходной путь мимо новой проверки. Удаляем: вход через Discord у
-- всех сохраняется, никто не теряет доступ.
DELETE FROM passkeys;

ALTER TABLE passkeys DROP COLUMN IF EXISTS public_key;
ALTER TABLE passkeys DROP COLUMN IF EXISTS counter;

-- Учётные данные целиком в том виде, в каком их держит webauthn-rs: там и
-- открытый ключ, и счётчик, и флаги — хранить их по кусочкам значит
-- пересобирать структуру библиотеки руками при каждом её обновлении.
ALTER TABLE passkeys ADD COLUMN IF NOT EXISTS credential JSONB NOT NULL;
ALTER TABLE passkeys ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMPTZ;

-- Состояние между /options и /verify. Раньше хранился только challenge, а
-- проверять нужно ещё и политику: user verification, допустимые ключи,
-- привязку к пользователю.
DROP TABLE IF EXISTS passkey_challenges;

CREATE TABLE IF NOT EXISTS webauthn_states (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- NULL для входа: кто именно входит, известно только после проверки.
    user_id    UUID REFERENCES users(id) ON DELETE CASCADE,
    kind       TEXT NOT NULL CHECK (kind IN ('register', 'login')),
    state      JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS webauthn_states_expires_idx ON webauthn_states (expires_at);
