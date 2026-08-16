-- Флаги целостности: что лаунчер нашёл в игровом каталоге перед запуском.
--
-- Это клиентский сигнал, а не доказательство: лаунчер открыт, свой билд
-- отправит что угодно. Поэтому таблица нужна для ручного разбора, и никакой
-- автоматики по ней не строится — автобан по такому репорту даёт ложные
-- срабатывания на честных (битый диск, антивирус) и ноль эффекта против того,
-- ради кого делался.
CREATE TABLE IF NOT EXISTS integrity_flags (
    id            BIGSERIAL PRIMARY KEY,
    at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id     UUID,
    build_id      UUID,
    build_version TEXT NOT NULL DEFAULT '',
    launcher_version TEXT NOT NULL DEFAULT '',

    -- extra_file | modified_file | missing_file | forbidden_optional_mod
    kind          TEXT NOT NULL,
    -- Путь внутри инстанса либо имя мода.
    subject       TEXT NOT NULL,
    detail        TEXT,
    -- Лаунчер убрал находку сам (удалил лишний файл).
    repaired      BOOLEAN NOT NULL DEFAULT FALSE,

    -- Разобрано админом: флаг перестаёт мозолить глаза, но остаётся в истории.
    reviewed_at   TIMESTAMPTZ,
    reviewed_by   UUID REFERENCES users(id) ON DELETE SET NULL
);

-- Хранятся бессрочно: строки лёгкие, а смысл в том, чтобы через полгода
-- увидеть, что это уже третий такой случай у одного игрока.
CREATE INDEX IF NOT EXISTS integrity_flags_user_idx ON integrity_flags (user_id, at DESC);
CREATE INDEX IF NOT EXISTS integrity_flags_open_idx ON integrity_flags (at DESC)
    WHERE reviewed_at IS NULL;
