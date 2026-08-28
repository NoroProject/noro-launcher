UPDATE roles SET name = 'friend', display_name = 'Friend' WHERE name = 'vip';

-- A database seeded after 0002 never had 'vip', so create the role outright.
INSERT INTO roles (name, display_name, color, is_default, sort_order)
VALUES ('friend', 'Friend', '#F59E0B', FALSE, 50)
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission)
SELECT id, 'noro.optional.*' FROM roles WHERE name = 'friend'
ON CONFLICT DO NOTHING;
