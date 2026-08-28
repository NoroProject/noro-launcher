-- The master and LuckPerms keep separate role models on purpose; `lp_group` is
-- the only thing tying them together, and it is what the syncer matches on. NULL
-- means the role is not projected into the game at all.
--
-- `icon` is an icon name or a unicode glyph, not a file: it is drawn next to the
-- nickname both on the site and in in-game chat.

ALTER TABLE roles ADD COLUMN IF NOT EXISTS lp_group TEXT;
ALTER TABLE roles ADD COLUMN IF NOT EXISTS icon TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_roles_lp_group
    ON roles (lp_group) WHERE lp_group IS NOT NULL;
