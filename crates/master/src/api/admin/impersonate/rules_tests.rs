//! Ошибка здесь — это эскалация привилегий: админ надевает другого админа и
//! получает его права. Проверяется обе стороны — что можно и что нельзя.

use super::*;
use uuid::Uuid;

fn user(id: u128, perms: &[&str]) -> UserProfile {
    UserProfile {
        id: Uuid::from_u128(id),
        uuid: Uuid::nil(),
        username: format!("u{id}"),
        discord_id: None,
        discord_username: None,
        discord_avatar: None,
        skin_url: None,
        cape_url: None,
        roles: Vec::new(),
        permissions: perms.iter().map(|p| p.to_string()).collect(),
        permission_grants: Vec::new(),
        banned: false,
        is_local_account: false,
        can_play: true,
        is_root: false,
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
    // Права игроков выданы поимённо, у админов — шаблонами. Без сопоставления
    // по шаблонам правило запрещало бы почти всё.
    let actor = user(1, &["noro.admin.*"]);
    let target = user(2, &["noro.admin.users"]);
    assert!(can_impersonate(&actor, &target));
}

#[test]
fn an_admin_cannot_impersonate_another_admin() {
    // Тот самый случай, ради которого правило и существует.
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
    // Иначе действия админа прятались бы под видом impersonation.
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
    // У актора нет вообще ничего — надевать некого, даже пустого игрока:
    // само право impersonate проверяется отдельно, но подмножество тут пустое
    // и формально сходится.
    let actor = user(1, &[]);
    assert!(can_impersonate(&actor, &user(2, &[])));
    assert!(!can_impersonate(&actor, &user(3, &["noro.launcher.beta"])));
}
