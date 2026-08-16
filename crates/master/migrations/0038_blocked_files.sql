-- База запрещённых файлов.
--
-- Отдаётся внутри подписанного манифеста: иначе список подменяется на клиенте,
-- и вся затея теряет смысл. Приоритет выше всех правил путей — папка
-- ресурспаков не синхронизируется, но xray оттуда удаляется.
CREATE TABLE IF NOT EXISTS blocked_files (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Маска имени: *xray*. NULL — правило только по хешу.
    pattern    TEXT,
    -- Точный SHA1. NULL — правило только по маске.
    sha1       TEXT,
    reason     TEXT NOT NULL,
    -- delete | flag | block_launch
    action     TEXT NOT NULL DEFAULT 'delete',
    -- NULL — правило действует на всех серверах.
    server_id  UUID REFERENCES servers(id) ON DELETE CASCADE,
    -- Порядок важен: решает первое сработавшее правило, и более узкое ставится
    -- выше.
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Пустое правило матчило бы всё подряд и снесло игроку каталог.
    CONSTRAINT blocked_files_not_empty CHECK (pattern IS NOT NULL OR sha1 IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS blocked_files_scope_idx ON blocked_files (server_id, sort_order);
