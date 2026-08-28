-- The instances running one build: survival, anarchy, events. `servers.mc_host`
-- stays the address players connect to (usually a proxy); these rows are the
-- backends behind it, and each agent reports against exactly one of them.
CREATE TABLE IF NOT EXISTS game_servers (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id    UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    mc_host      TEXT NOT NULL DEFAULT '',
    mc_port      INT  NOT NULL DEFAULT 25565,
    -- Plain sha256, like admin_tokens: the secret is high-entropy, so no KDF is
    -- needed and lookups can hit the index directly. Plaintext is shown once, at
    -- issue time.
    token_hash   TEXT NOT NULL UNIQUE,
    sort_order   INT  NOT NULL DEFAULT 0,
    -- Only meaningful next to a fresh last_seen_at: a server that died keeps its
    -- last player count here forever.
    online       INT  NOT NULL DEFAULT 0,
    max_online   INT  NOT NULL DEFAULT 0,
    version      TEXT,
    last_seen_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_game_servers_server ON game_servers(server_id);
