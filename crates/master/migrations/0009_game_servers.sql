-- Игровые сервера, работающие на одной сборке.
--
-- Раньше карточка сборки была и адресом сервера: mc_host/mc_port лежали прямо
-- в `servers`. Но на одной сборке живёт несколько инстансов (сурвайвал,
-- анархия, ивент), а агенту нужно знать, за какой именно он отчитывается.
--
-- Адрес в `servers` остаётся адресом входа (обычно прокси), сюда пишутся
-- бэкенды, которые докладывают онлайн и раздают роли.
CREATE TABLE IF NOT EXISTS game_servers (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id    UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    mc_host      TEXT NOT NULL DEFAULT '',
    mc_port      INT  NOT NULL DEFAULT 25565,
    -- sha256 секрета: в базе только хеш, plaintext админ видит один раз при
    -- выдаче. Секрет высокоэнтропийный, поэтому обычного sha256 достаточно и
    -- поиск идёт прямо по индексу — как у admin_tokens.
    token_hash   TEXT NOT NULL UNIQUE,
    sort_order   INT  NOT NULL DEFAULT 0,
    -- Последний heartbeat. online без свежего last_seen_at не значит ничего:
    -- упавший сервер иначе навсегда остался бы с последним числом игроков.
    online       INT  NOT NULL DEFAULT 0,
    max_online   INT  NOT NULL DEFAULT 0,
    version      TEXT,
    last_seen_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_game_servers_server ON game_servers(server_id);
