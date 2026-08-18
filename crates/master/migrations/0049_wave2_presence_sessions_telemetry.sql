-- 0049_wave2_presence_sessions_telemetry.sql

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
