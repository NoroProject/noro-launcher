DO $$
DECLARE
    player_id UUID;
    helper_id UUID;
    mod_id UUID;
BEGIN
    SELECT id INTO player_id FROM roles WHERE name = 'player';
    
    INSERT INTO roles (name, display_name, color, is_default, sort_order, parent_id)
    VALUES ('helper', 'Хелпер', '#34d399', false, 70, player_id)
    RETURNING id INTO helper_id;

    INSERT INTO roles (name, display_name, color, is_default, sort_order, parent_id)
    VALUES ('moderator', 'Модератор', '#3b82f6', false, 80, helper_id)
    RETURNING id INTO mod_id;

    -- Helper permissions
    INSERT INTO role_permissions (role_id, permission) VALUES
    (helper_id, 'noro.mod.punish.view'),
    (helper_id, 'noro.mod.punish.warn'),
    (helper_id, 'noro.mod.punish.mute'),
    (helper_id, 'noro.mod.cases.view'),
    (helper_id, 'noro.mod.cases.claim'),
    (helper_id, 'noro.mod.cases.resolve'),
    (helper_id, 'noro.mod.cases.chat'),
    (helper_id, 'noro.mod.cases.inventory'),
    (helper_id, 'noro.mod.cases.watch'),
    (helper_id, 'noro.mod.cases.client'),
    (helper_id, 'noro.mod.vanish.case_only');

    -- Moderator permissions (inherits helper, so only need to add extras)
    INSERT INTO role_permissions (role_id, permission) VALUES
    (mod_id, 'noro.mod.freeze'),
    (mod_id, 'noro.admin.users.view'),
    (mod_id, 'noro.admin.integrity.view'),
    (mod_id, 'noro.admin.integrity.review'),
    (mod_id, 'noro.admin.users.journal'),
    (mod_id, 'noro.mod.punish.ban'),
    (mod_id, 'noro.mod.punish.server_ban'),
    (mod_id, 'noro.mod.punish.revoke'),
    (mod_id, 'noro.mod.vanish.use'),
    (mod_id, 'noro.mod.vanish.silent_join');
END $$;
