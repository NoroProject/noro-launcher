-- Бандлы логов, присланные лаунчерами.
--
-- Сам архив лежит в FileStore и адресуется по sha1; здесь только метаданные.
-- Так одинаковые бандлы не дублируются на диске, а удаление строки оставляет
-- осиротевший blob, который подберёт files/gc.rs.
CREATE TABLE IF NOT EXISTS support_bundles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id   UUID,

    -- Что игрок написал сам, отправляя бандл.
    note        TEXT NOT NULL DEFAULT '',
    -- Отправлен добровольно (кнопка «Сообщить о проблеме») либо по запросу
    -- админа. Добровольные игрок может удалить сам, собранные по запросу — нет.
    voluntary   BOOLEAN NOT NULL DEFAULT TRUE,

    file_sha1   TEXT NOT NULL,
    size        BIGINT NOT NULL,
    -- 30 дней: дальше архив бесполезен, а хранить логи вечно незачем.
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS support_bundles_user_idx ON support_bundles (user_id, at DESC);
CREATE INDEX IF NOT EXISTS support_bundles_expires_idx ON support_bundles (expires_at);
