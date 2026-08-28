-- Build access is now a permission (`noro.build.<server>.<build>`) and the
-- launcher only lists versions the player holds one for. `player` had no
-- permissions at all, so without this row the rollout would lock everyone out at
-- once. The wildcard keeps the old behaviour; narrow it in the admin panel.
INSERT INTO role_permissions (role_id, permission)
SELECT id, 'noro.build.*' FROM roles WHERE name = 'player'
ON CONFLICT DO NOTHING;
