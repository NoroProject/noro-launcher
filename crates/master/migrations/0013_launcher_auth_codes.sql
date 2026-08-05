-- Мастер отдавал токены лаунчеру POST-запросом на 127.0.0.1:{port}, то есть на
-- собственный localhost. Совпадало это только когда мастер запущен на машине
-- игрока; с боевого сервера запрос уходил внутрь контейнера, и лаунчер вечно
-- висел в «ожидании».
--
-- Теперь браузер игрока сам открывает loopback лаунчера, получив одноразовый
-- код, а за токенами лаунчер идёт к мастеру по HTTPS. Токены не попадают ни в
-- историю браузера, ни в логи прокси — в URL едет только код с коротким TTL.
CREATE TABLE IF NOT EXISTS launcher_auth_codes (
    code          UUID PRIMARY KEY,
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token  UUID NOT NULL,
    refresh_token UUID NOT NULL,
    expires_at    TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS launcher_auth_codes_expires ON launcher_auth_codes (expires_at);
