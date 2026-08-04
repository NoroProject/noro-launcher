-- Тип игрового сервера и переезд адреса со сборки.
--
-- Адрес лежал в `servers` и дублировал адреса игровых серверов. Теперь он
-- живёт только здесь: прокси — точка входа для игрока, бэкенды — то, где
-- крутится агент и считается онлайн.
ALTER TABLE game_servers ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'server';

-- У сборок, которым игровые сервера ещё не завели, адрес превращается в
-- обычный сервер: раньше он был единственным, значит на него и заходили.
--
-- Секрет для такой записи неизвестен никому: в token_hash кладётся значение,
-- прообраз которого не существует. Агент по нему не подключится, пока админ
-- не перевыпустит секрет — это честнее, чем выдать общий предсказуемый.
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
