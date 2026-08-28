-- Role inheritance: a child also gets the parent's permissions.
--
-- A single parent rather than a list. Multiple inheritance would need conflict
-- resolution rules, and there is nothing to resolve here — a permission is
-- either granted or it isn't. Chains like vip -> vip+ -> vip++ cover the cases
-- this exists for.
--
-- SET NULL rather than CASCADE: deleting a parent must not take child roles with
-- it, along with their own permissions and everyone holding them.
--
-- Cycles (A -> B -> A) are not blocked here; the API checks on reparent. The
-- recursive traversals use UNION, which terminates on a cycle regardless, so bad
-- data can't hang the permission resolver.

ALTER TABLE roles ADD COLUMN IF NOT EXISTS parent_id uuid
    REFERENCES roles(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_roles_parent
    ON roles (parent_id) WHERE parent_id IS NOT NULL;
