-- Scopes a permission to one build: NULL is global, otherwise it only applies on
-- that server. Same idea as a LuckPerms context, minus the general machinery —
-- the build is the only dimension we have.

ALTER TABLE role_permissions ADD COLUMN IF NOT EXISTS server_id uuid
    REFERENCES servers(id) ON DELETE CASCADE;
ALTER TABLE user_permissions ADD COLUMN IF NOT EXISTS server_id uuid
    REFERENCES servers(id) ON DELETE CASCADE;

-- The key has to tell the same permission apart across contexts. NULL is not
-- equal to itself in Postgres, so global rows need their own partial index —
-- a plain unique constraint would let them be inserted twice.
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

-- Nodes the agent has seen on its server, for autocomplete in the admin panel.
-- Mods bring their own, so there is no list to ship ahead of time.
CREATE TABLE IF NOT EXISTS permission_nodes (
    server_id  uuid NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    node       text NOT NULL,
    seen_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (server_id, node)
);

CREATE INDEX IF NOT EXISTS idx_permission_nodes_server ON permission_nodes (server_id);
