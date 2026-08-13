-- Доступ к сборкам стал правом (`noro.build.<server>.<build>`), и лаунчер
-- показывает только те версии, на которые право есть.
--
-- Без этой строки выкат отобрал бы доступ у всех разом: у роли `player` базовых
-- прав нет вовсе, вход на нелимитные серверы держался на логике `limited=false`.
-- Wildcard сохраняет прежнее поведение — все видят все сборки; сузить его можно
-- в админке, заменив на `noro.build.<server>.*` или на конкретные версии.
INSERT INTO role_permissions (role_id, permission)
SELECT id, 'noro.build.*' FROM roles WHERE name = 'player'
ON CONFLICT DO NOTHING;
