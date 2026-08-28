-- Which launcher build each player is on, so "it doesn't work for me" can be
-- answered. One row per player: the current state is what matters, not history.
CREATE TABLE IF NOT EXISTS launcher_clients (
    user_id      UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    version      TEXT NOT NULL DEFAULT '',
    platform     TEXT NOT NULL DEFAULT '',
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_launcher_clients_version ON launcher_clients(version);
