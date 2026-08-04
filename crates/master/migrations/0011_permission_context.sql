-- Контекст сервера у прав и каталог известных узлов.
--
-- До сих пор право действовало везде. Теперь у него может быть сборка:
-- NULL — глобально, иначе только на этом сервере. Тот же смысл, что context
-- у LuckPerms, но без общей машинерии контекстов: сборка — единственное
-- измерение, которое здесь есть.

ALTER TABLE role_permissions ADD COLUMN IF NOT EXISTS server_id uuid
    REFERENCES servers(id) ON DELETE CASCADE;
ALTER TABLE user_permissions ADD COLUMN IF NOT EXISTS server_id uuid
    REFERENCES servers(id) ON DELETE CASCADE;

-- Первичный ключ должен различать одно и то же право в разных контекстах.
-- NULL в PostgreSQL не равен сам себе, поэтому уникальность глобальных прав
-- держим отдельным частичным индексом.
ALTER TABLE role_permissions DROP CONSTRAINT IF EXISTS role_permissions_pkey;
CREATE UNIQUE INDEX IF NOT EXISTS role_permissions_scoped
    ON role_permissions (role_id, permission, server_id) WHERE server_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS role_permissions_global
    ON role_permissions (role_id, permission) WHERE server_id IS NULL;

ALTER TABLE user_permissions DROP CONSTRAINT IF EXISTS user_permissions_pkey;
CREATE UNIQUE INDEX IF NOT EXISTS user_permissions_scoped
    ON user_permissions (user_id, permission, server_id) WHERE server_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS user_permissions_global
    ON user_permissions (user_id, permission) WHERE server_id IS NULL;

-- Каталог узлов, которые агент видит на своём сервере. Нужен админке для
-- автодополнения: перечислить их заранее неоткуда, их приносят сами моды.
CREATE TABLE IF NOT EXISTS permission_nodes (
    server_id  uuid NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    node       text NOT NULL,
    seen_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (server_id, node)
);

CREATE INDEX IF NOT EXISTS idx_permission_nodes_server ON permission_nodes (server_id);
