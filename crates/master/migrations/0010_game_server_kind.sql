-- `kind` splits proxies (the address players connect to) from backends (where
-- the agent runs and online is counted). The address now lives only here.
ALTER TABLE game_servers ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'server';

-- Builds that never got a game_server row keep their old address as a backend.
--
-- The token_hash below has no preimage — nobody, including us, knows a secret
-- that hashes to it. The agent stays locked out until an admin reissues one,
-- which beats seeding a predictable shared secret.
INSERT INTO game_servers (server_id, name, mc_host, mc_port, token_hash, kind)
SELECT s.id,
       s.name,
       s.mc_host,
       s.mc_port,
       md5(random()::text || clock_timestamp()::text || s.id::text),
       'server'
FROM servers s
WHERE NOT EXISTS (SELECT 1 FROM game_servers g WHERE g.server_id = s.id);

ALTER TABLE servers DROP COLUMN IF EXISTS mc_host;
ALTER TABLE servers DROP COLUMN IF EXISTS mc_port;
