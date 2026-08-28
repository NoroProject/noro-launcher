-- Base roles. `player` is handed out automatically on first login.

INSERT INTO roles (name, display_name, color, is_default, sort_order)
VALUES
    ('player', 'Игрок', '#9CA3AF', TRUE, 0),
    ('vip',    'VIP',   '#F59E0B', FALSE, 50),
    ('admin',  'Админ', '#5865F2', FALSE, 100)
ON CONFLICT (name) DO NOTHING;

-- `player` deliberately gets nothing: entry to non-limited servers is decided by
-- `servers.limited = false`, not by a grant.
INSERT INTO role_permissions (role_id, permission)
SELECT id, 'noro.optional.*' FROM roles WHERE name = 'vip'
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission)
SELECT id, '*' FROM roles WHERE name = 'admin'
ON CONFLICT DO NOTHING;
