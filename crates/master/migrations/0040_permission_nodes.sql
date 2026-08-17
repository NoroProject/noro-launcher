-- Права разложены на узлы: вместо «noro.admin.users» теперь ветка
-- «noro.admin.users.view/.edit/.roles/…». Уже выданные права надо перевести,
-- иначе после обновления модератор молча теряет доступ ко всему сразу.
--
-- Правило перевода: тому, у кого была вся область, отдаётся вся ветка (`.*`).
-- Сужать чужие права миграцией нельзя — это решение админа, а не апдейта.

-- Области, ставшие ветками.
UPDATE role_permissions SET permission = permission || '.*'
 WHERE permission IN (
   'noro.admin.users', 'noro.admin.servers', 'noro.admin.builds',
   'noro.admin.news', 'noro.admin.rules', 'noro.admin.roles',
   'noro.admin.launcher', 'noro.admin.wrapper', 'noro.admin.settings',
   'noro.admin.integrity', 'noro.admin.blocklist', 'noro.admin.capes',
   'noro.admin.mods', 'noro.admin.translations'
 );

UPDATE user_permissions SET permission = permission || '.*'
 WHERE permission IN (
   'noro.admin.users', 'noro.admin.servers', 'noro.admin.builds',
   'noro.admin.news', 'noro.admin.rules', 'noro.admin.roles',
   'noro.admin.launcher', 'noro.admin.wrapper', 'noro.admin.settings',
   'noro.admin.integrity', 'noro.admin.blocklist', 'noro.admin.capes',
   'noro.admin.mods', 'noro.admin.translations'
 );

-- Переехавшие узлы.
UPDATE role_permissions SET permission = 'noro.admin.users.impersonate'
 WHERE permission = 'noro.admin.impersonate';
UPDATE user_permissions SET permission = 'noro.admin.users.impersonate'
 WHERE permission = 'noro.admin.impersonate';

UPDATE role_permissions SET permission = 'noro.admin.support.force'
 WHERE permission = 'noro.admin.support.logs.force';
UPDATE user_permissions SET permission = 'noro.admin.support.force'
 WHERE permission = 'noro.admin.support.logs.force';

-- Старое право «банить» разложено на виды наказаний. Байпас рамок правила и
-- вечные сроки не выдаются: их смысл появился только сейчас, и раздавать их
-- задним числом тем, у кого был обычный бан, нельзя.
INSERT INTO role_permissions (role_id, permission)
SELECT role_id, node
  FROM role_permissions
  CROSS JOIN unnest(ARRAY[
    'noro.mod.punish.view', 'noro.mod.punish.warn', 'noro.mod.punish.mute',
    'noro.mod.punish.ban', 'noro.mod.punish.server_ban', 'noro.mod.punish.revoke'
  ]) AS node
 WHERE permission = 'noro.mod.users.ban'
ON CONFLICT DO NOTHING;

INSERT INTO user_permissions (user_id, permission)
SELECT user_id, node
  FROM user_permissions
  CROSS JOIN unnest(ARRAY[
    'noro.mod.punish.view', 'noro.mod.punish.warn', 'noro.mod.punish.mute',
    'noro.mod.punish.ban', 'noro.mod.punish.server_ban', 'noro.mod.punish.revoke'
  ]) AS node
 WHERE permission = 'noro.mod.users.ban'
ON CONFLICT DO NOTHING;

DELETE FROM role_permissions WHERE permission = 'noro.mod.users.ban';
DELETE FROM user_permissions WHERE permission = 'noro.mod.users.ban';
