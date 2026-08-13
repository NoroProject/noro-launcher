-- Какой лаунчер у какого игрока.
--
-- Раньше версия клиента не доходила до мастера вообще: сказать, кто остался на
-- старой сборке и почему у него «не работает», было нечем. Одна строка на
-- игрока — история не нужна, важно текущее состояние.
CREATE TABLE IF NOT EXISTS launcher_clients (
    user_id      UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    version      TEXT NOT NULL DEFAULT '',
    platform     TEXT NOT NULL DEFAULT '',
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_launcher_clients_version ON launcher_clients(version);
