-- Базовые роли. player выдаётся всем автоматически.

INSERT INTO roles (name, display_name, color, is_default, sort_order)
VALUES
    ('player', 'Игрок', '#9CA3AF', TRUE, 0),
    ('vip',    'VIP',   '#F59E0B', FALSE, 50),
    ('admin',  'Админ', '#5865F2', FALSE, 100)
ON CONFLICT (name) DO NOTHING;

-- player: доступ к публичным серверам и опциональным модам (нелимитным — без прав вообще).
-- Здесь только право входа на все нелимитные серверы покрывается логикой (limited=false),
-- поэтому базовых прав у player нет. VIP получает все опциональные моды.
INSERT INTO role_permissions (role_id, permission)
SELECT id, 'noro.optional.*' FROM roles WHERE name = 'vip'
ON CONFLICT DO NOTHING;

-- admin: полный доступ.
INSERT INTO role_permissions (role_id, permission)
SELECT id, '*' FROM roles WHERE name = 'admin'
ON CONFLICT DO NOTHING;
