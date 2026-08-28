//! A bug here is privilege escalation, so both directions are covered: what is
//! allowed and what must be refused.

use super::*;
use uuid::Uuid;

fn user(id: u128, perms: &[&str]) -> UserProfile {
    UserProfile {
        id: Uuid::from_u128(id),
        uuid: Uuid::nil(),
        username: format!("u{id}"),
        identities: Vec::new(),
        skin_url: None,
        skin_slim: false,
        cape_url: None,
        roles: Vec::new(),
        permissions: perms.iter().map(|p| p.to_string()).collect(),
        permission_grants: Vec::new(),
        banned: false,
        ban_reason: None,
        created_at: None,
        last_login_at: None,
        is_local_account: false,
        can_play: true,
        is_root: false,
        hide_from_online: false,
        frozen: false,
        freeze_info: None,
        silent_join: false,
    }
}

#[test]
fn an_admin_can_impersonate_an_ordinary_player() {
    let actor = user(1, &["noro.admin.*", "noro.admin.impersonate"]);
    let target = user(2, &[]);
    assert!(can_impersonate(&actor, &target));
}

#[test]
fn wildcards_cover_named_permissions() {
    // Players are granted by name, admins by pattern. Matching literally would
    // refuse nearly everything.
    let actor = user(1, &["noro.admin.*"]);
    let target = user(2, &["noro.admin.users"]);
    assert!(can_impersonate(&actor, &target));
}

#[test]
fn an_admin_cannot_impersonate_another_admin() {
    // The case the whole rule exists for.
    let actor = user(1, &["noro.admin.users", "noro.admin.impersonate"]);
    let target = user(2, &["noro.admin.builds"]);
    assert!(!can_impersonate(&actor, &target));
}

#[test]
fn a_superadmin_can_impersonate_anyone_but_root() {
    let actor = user(1, &["*"]);
    assert!(can_impersonate(&actor, &user(2, &["noro.admin.*"])));

    let mut root = user(3, &[]);
    root.is_root = true;
    assert!(!can_impersonate(&actor, &root));
}

#[test]
fn nobody_can_impersonate_themselves() {
    // Otherwise an admin's own actions could hide behind an impersonation.
    let actor = user(1, &["*"]);
    assert!(!can_impersonate(&actor, &actor));
}

#[test]
fn a_single_extra_permission_is_enough_to_refuse() {
    let actor = user(1, &["noro.admin.users"]);
    let target = user(2, &["noro.admin.users", "noro.launcher.beta"]);
    assert!(!can_impersonate(&actor, &target));
}

#[test]
fn a_player_without_permissions_cannot_be_used_to_reach_further() {
    // With nothing on either side the subset check trivially passes; holding
    // the impersonate permission at all is checked elsewhere.
    let actor = user(1, &[]);
    assert!(can_impersonate(&actor, &user(2, &[])));
    assert!(!can_impersonate(&actor, &user(3, &["noro.launcher.beta"])));
}
