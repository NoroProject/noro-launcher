-- 0054_chat_filters_rule_code_and_triggers.sql

ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS rule_code TEXT;

-- Гарантируем наличие всех колонок в restart_schedules для БД,
-- где таблица была создана до миграции 0051 (CREATE TABLE IF NOT EXISTS пропустила её).
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS cron_expr TEXT;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS at_times TEXT[];
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS interval_minutes INT;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS notice_minutes INT NOT NULL DEFAULT 5;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS online_policy TEXT NOT NULL DEFAULT 'warn_and_go';
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS max_defer_minutes INT NOT NULL DEFAULT 30;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS last_run_at TIMESTAMPTZ;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS next_run_at TIMESTAMPTZ;

ALTER TABLE player_freezes ADD COLUMN IF NOT EXISTS released_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS automod_triggers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game_server_id UUID REFERENCES game_servers(id) ON DELETE SET NULL,
    rule_type TEXT NOT NULL,
    action TEXT NOT NULL,
    trigger_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_automod_triggers_created ON automod_triggers(created_at DESC);

-- server_telemetry_hourly могла быть создана старым runtime DDL с другой схемой.
-- Пересоздаём таблицу, если она не содержит колонку bucket_hour (данные несущественны,
-- таблица заполняется cleanup-задачей из сырых точек).
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'server_telemetry_hourly' AND column_name = 'bucket_hour'
    ) THEN
        DROP TABLE IF EXISTS server_telemetry_hourly;
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS server_telemetry_hourly (
    id BIGSERIAL PRIMARY KEY,
    game_server_id UUID NOT NULL REFERENCES game_servers(id) ON DELETE CASCADE,
    bucket_hour TIMESTAMPTZ NOT NULL,
    tps_min DOUBLE PRECISION,
    tps_max DOUBLE PRECISION,
    tps_avg DOUBLE PRECISION,
    mspt_avg DOUBLE PRECISION,
    online_max INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_telemetry_hourly UNIQUE (game_server_id, bucket_hour)
);

CREATE INDEX IF NOT EXISTS idx_telemetry_hourly_server_bucket ON server_telemetry_hourly(game_server_id, bucket_hour DESC);
