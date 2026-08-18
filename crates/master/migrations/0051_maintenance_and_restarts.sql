-- 0051_maintenance_and_restarts.sql

ALTER TABLE game_servers ADD COLUMN IF NOT EXISTS maintenance BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE game_servers ADD COLUMN IF NOT EXISTS maintenance_reason TEXT;

CREATE TABLE IF NOT EXISTS restart_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_server_id UUID NOT NULL REFERENCES game_servers(id) ON DELETE CASCADE,
    cron_expr TEXT,
    at_times TEXT[],
    interval_minutes INT,
    notice_minutes INT NOT NULL DEFAULT 5,
    online_policy TEXT NOT NULL DEFAULT 'warn_and_go',
    max_defer_minutes INT NOT NULL DEFAULT 30,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_restart_schedules_server ON restart_schedules(game_server_id);
