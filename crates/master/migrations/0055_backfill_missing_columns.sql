-- 0055_backfill_missing_columns.sql
-- Бэкфилл колонок и таблиц для БД, где миграции 0049-0054 частично пропустили
-- объекты из-за CREATE TABLE IF NOT EXISTS на уже существующих таблицах.
-- Все операции идемпотентны.

-- === 0049: presence, sessions, telemetry ===
ALTER TABLE users ADD COLUMN IF NOT EXISTS hide_from_online BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS player_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game_server_id UUID NOT NULL REFERENCES game_servers(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    end_reason TEXT
);
CREATE INDEX IF NOT EXISTS idx_player_sessions_user_id ON player_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_player_sessions_game_server_id ON player_sessions(game_server_id);
CREATE INDEX IF NOT EXISTS idx_player_sessions_open ON player_sessions(game_server_id) WHERE ended_at IS NULL;

CREATE TABLE IF NOT EXISTS player_activity_days (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    day DATE NOT NULL,
    minutes INT NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, day)
);

CREATE TABLE IF NOT EXISTS server_telemetry (
    id BIGSERIAL PRIMARY KEY,
    game_server_id UUID NOT NULL REFERENCES game_servers(id) ON DELETE CASCADE,
    tps DOUBLE PRECISION,
    mspt DOUBLE PRECISION,
    heap_used_mb BIGINT,
    heap_max_mb BIGINT,
    online_players INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_server_telemetry_server_created ON server_telemetry(game_server_id, created_at DESC);

-- === 0050: freezes, reports ===
CREATE TABLE IF NOT EXISTS player_freezes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    released_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_player_freezes_user ON player_freezes(user_id);
CREATE INDEX IF NOT EXISTS idx_player_freezes_active ON player_freezes(user_id) WHERE released_at IS NULL;
ALTER TABLE player_freezes ADD COLUMN IF NOT EXISTS released_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS player_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game_server_id UUID NOT NULL REFERENCES game_servers(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    world TEXT,
    x DOUBLE PRECISION,
    y DOUBLE PRECISION,
    z DOUBLE PRECISION,
    status TEXT NOT NULL DEFAULT 'open',
    claimed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    claimed_at TIMESTAMPTZ,
    resolution TEXT,
    punishment_id UUID REFERENCES punishments(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_player_reports_status ON player_reports(status);
CREATE INDEX IF NOT EXISTS idx_player_reports_created ON player_reports(created_at DESC);

CREATE TABLE IF NOT EXISTS pending_report_feedbacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_username TEXT NOT NULL,
    resolution TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_pending_report_feedbacks_user ON pending_report_feedbacks(user_id);

-- === 0051: maintenance, restart_schedules ===
ALTER TABLE game_servers ADD COLUMN IF NOT EXISTS maintenance BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE game_servers ADD COLUMN IF NOT EXISTS maintenance_reason TEXT;

ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS cron_expr TEXT;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS at_times TEXT[];
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS interval_minutes INT;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS notice_minutes INT NOT NULL DEFAULT 5;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS online_policy TEXT NOT NULL DEFAULT 'warn_and_go';
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS max_defer_minutes INT NOT NULL DEFAULT 30;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS active BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS last_run_at TIMESTAMPTZ;
ALTER TABLE restart_schedules ADD COLUMN IF NOT EXISTS next_run_at TIMESTAMPTZ;

-- === 0052: vanish ===
CREATE TABLE IF NOT EXISTS player_vanish (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE users ADD COLUMN IF NOT EXISTS silent_join BOOLEAN NOT NULL DEFAULT FALSE;

-- === 0053: chat_filters ===
CREATE TABLE IF NOT EXISTS chat_filters (
    filter_type TEXT PRIMARY KEY,
    mode TEXT NOT NULL DEFAULT 'deny',
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    config JSONB NOT NULL DEFAULT '{}'::jsonb
);

INSERT INTO chat_filters (filter_type, mode, enabled, config)
VALUES
    ('ad', 'punish', true, '{"whitelist":["example.com"]}'::jsonb),
    ('word', 'deny', true, '{"words":["badword"]}'::jsonb),
    ('caps', 'deny', true, '{"threshold":0.6,"min_length":6}'::jsonb),
    ('flood', 'deny', true, '{"max_messages":3,"window_secs":4}'::jsonb)
ON CONFLICT (filter_type) DO NOTHING;

-- === 0054: rule_code, automod_triggers, server_telemetry_hourly ===
ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS rule_code TEXT;

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
