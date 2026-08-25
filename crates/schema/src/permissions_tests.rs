use super::*;

#[test]
fn wildcard_matching() {
    assert!(permission_matches("*", "anything.here"));
    assert!(permission_matches("noro.server.*", "noro.server.hitech"));
    assert!(permission_matches(
        "noro.server.*",
        "noro.server.hitech.join"
    ));
    assert!(permission_matches(
        "noro.server.hitech.join",
        "noro.server.hitech.join"
    ));
    assert!(!permission_matches(
        "noro.server.*",
        "noro.admin.users.view"
    ));
    assert!(!permission_matches(
        "noro.server.hitech",
        "noro.server.hitech2"
    ));
}

/// Право на один сервер не должно открывать соседний: именно это отделяет
/// закрытую сборку от общедоступной.
#[test]
fn server_grant_does_not_leak_to_another_server() {
    let perms = ["noro.server.hitech.join"];
    assert!(any_permission_matches(perms, "noro.server.hitech.join"));
    assert!(!any_permission_matches(perms, "noro.server.vanilla.join"));
}

/// Шаблон обрывается на границе сегмента, а не по префиксу строки: иначе
/// `noro.admin.*` выдал бы права на всё, что просто начинается так же.
#[test]
fn wildcard_stops_at_the_segment_boundary() {
    assert!(!permission_matches("noro.admin.*", "noro.adminx.users"));
    assert!(!permission_matches("noro.server.*", "noro.servers.list"));
    assert!(permission_matches("noro.admin.*", "noro.admin.users.view"));
}

/// `*` поддерживается только как одиночный шаблон и как суффикс `.*`.
#[test]
fn other_star_placements_match_nothing() {
    assert!(!permission_matches("noro.*.users", "noro.admin.users"));
    assert!(!permission_matches("*.users", "noro.admin.users"));
    assert!(!permission_matches("noro.admin*", "noro.admin.users"));
}

#[test]
fn superadmin_covers_every_permission() {
    let perms = [PERM_SUPERADMIN];
    assert!(any_permission_matches(perms, PERM_WRAPPER_FILES));
    assert!(any_permission_matches(perms, &perm_server_join("hitech")));
}

#[test]
fn empty_permission_set_grants_nothing() {
    let empty: [&str; 0] = [];
    assert!(!any_permission_matches(empty, PERM_USERS_VIEW));
}

/// Управление игровой машиной — отдельная ветка, и админ серверов её не
/// получает: запись файла плюс рестарт это фактически рут на машине.
#[test]
fn server_admin_does_not_imply_machine_access() {
    let perms = [PERM_SERVERS_EDIT];
    assert!(!any_permission_matches(perms, PERM_WRAPPER_FILES));
    assert!(any_permission_matches([PERM_ADMIN_ALL], PERM_WRAPPER_FILES));
}

/// Ветка игрока выдаётся целиком одной строкой — ради этого узлы и разложены
/// по общему префиксу.
#[test]
fn a_branch_can_be_granted_at_once() {
    let perms = ["noro.admin.users.*"];
    assert!(any_permission_matches(perms, PERM_USERS_VIEW));
    assert!(any_permission_matches(perms, PERM_USERS_NOTES_WRITE));
    assert!(!any_permission_matches(perms, PERM_PUNISH_BAN));
}

/// Хелпер выдаёт мут, но не бан: виды наказаний — разные узлы.
#[test]
fn punishment_kinds_are_separate_nodes() {
    let helper = [PERM_PUNISH_MUTE, PERM_PUNISH_WARN];
    assert!(any_permission_matches(helper, &perm_punish("mute")));
    assert!(!any_permission_matches(helper, &perm_punish("ban")));
    assert!(!any_permission_matches(helper, PERM_PUNISH_BYPASS));
}

/// Три уровня выдачи для сборок: всё, все сборки сервера, одна версия.
#[test]
fn build_access_supports_three_wildcard_levels() {
    let node = perm_build_access("srv1", "b7");
    assert_eq!(node, "noro.build.srv1.b7");

    assert!(any_permission_matches(["noro.build.*"], &node));
    assert!(any_permission_matches(["noro.build.srv1.*"], &node));
    assert!(any_permission_matches([node.as_str()], &node));
    assert!(any_permission_matches([PERM_SUPERADMIN], &node));
}

/// Выдача на одну сборку не должна открывать соседнюю — ради этого всё и
/// затевалось: тестер получает только ту версию, что проверяет.
#[test]
fn build_access_does_not_leak_between_builds_or_servers() {
    let granted = perm_build_access("srv1", "b7");

    assert!(!any_permission_matches(
        [granted.as_str()],
        &perm_build_access("srv1", "b8")
    ));
    assert!(!any_permission_matches(
        ["noro.build.srv1.*"],
        &perm_build_access("srv2", "b7")
    ));
}

/// Реестр — единственный источник узлов, и дубль в нём означал бы две разные
/// подсказки на одно и то же право.
#[test]
fn the_registry_has_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for node in ALL_NODES {
        assert!(seen.insert(node.name), "duplicate node: {}", node.name);
        assert!(node.name.starts_with("noro."), "odd node: {}", node.name);
    }
}

/// Полный админский шаблон обязан покрывать каждый админский узел: он и
/// выдаётся как «полный доступ».
#[test]
fn admin_wildcard_covers_every_admin_node() {
    for node in ALL_NODES {
        if node.name.starts_with("noro.admin.") {
            assert!(
                permission_matches(PERM_ADMIN_ALL, node.name),
                "{} is not covered",
                node.name
            );
        }
    }
}
