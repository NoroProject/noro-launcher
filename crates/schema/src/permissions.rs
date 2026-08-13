//! Система прав на glob-строках.

/// Право — строка вида `noro.server.hitech.join`. Поддерживается `*` как
/// суффиксный wildcard (`noro.server.*`) и одиночный `*` (суперадмин).
pub type Permission = String;

/// `noro.server.*` матчит `noro.server.hitech` и `noro.server.hitech.join`.
/// `*` матчит всё.
pub fn permission_matches(pattern: &str, target: &str) -> bool {
    if pattern == "*" || pattern == target {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return target == prefix || target.starts_with(&format!("{prefix}."));
    }
    false
}

/// Проверка набора прав против требуемого.
pub fn any_permission_matches<'a, I>(perms: I, required: &str) -> bool
where
    I: IntoIterator<Item = &'a str>,
{
    perms.into_iter().any(|p| permission_matches(p, required))
}

// --- Встроенные константы прав -------------------------------------------------

pub const PERM_SUPERADMIN: &str = "*";

pub const PERM_ADMIN_ALL: &str = "noro.admin.*";
pub const PERM_ADMIN_USERS: &str = "noro.admin.users";
pub const PERM_ADMIN_SERVERS: &str = "noro.admin.servers";
pub const PERM_ADMIN_BUILDS: &str = "noro.admin.builds";
pub const PERM_ADMIN_NEWS: &str = "noro.admin.news";
pub const PERM_ADMIN_ROLES: &str = "noro.admin.roles";
pub const PERM_ADMIN_LAUNCHER: &str = "noro.admin.launcher";
/// Управление игровой машиной через ServerWrapper: файлы, конфиги, консоль,
/// питание, бэкапы. Отдельно от `noro.admin.servers` намеренно — запись файла
/// плюс рестарт это фактически рут на сервере, а не правка карточки в админке.
pub const PERM_ADMIN_WRAPPER: &str = "noro.admin.wrapper";

/// Уборка неиспользуемых объектов хранилища. Отдельно от остальных админских
/// прав: операция удаляет файлы с диска, и восстановить их можно только
/// перезаливкой сборки.
pub const PERM_ADMIN_STORAGE: &str = "noro.admin.storage";

/// Скачивание дампа БД. Отдельное право: в дампе лежат все пользователи,
/// их привязки и токены сессий — это самый чувствительный объект в системе.
pub const PERM_ADMIN_BACKUP: &str = "noro.admin.backup";

pub const PERM_MOD_USERS_BAN: &str = "noro.mod.users.ban";

pub const PERM_LAUNCHER_BETA: &str = "noro.launcher.beta";

/// Право на вход на конкретный сервер.
pub fn perm_server_join(server_id: &str) -> String {
    format!("noro.server.{server_id}.join")
}

/// Право на конкретный опциональный мод.
pub fn perm_optional_mod(server_id: &str, mod_name: &str) -> String {
    format!("noro.optional.{server_id}.{mod_name}")
}

/// Право на доступ к конкретной сборке.
///
/// Сервер идёт отдельным сегментом, чтобы работали три уровня выдачи:
/// `noro.build.*` — все сборки, `noro.build.<server>.*` — все сборки одного
/// сервера, полный узел — одна версия. Так тестеру выдаётся ровно та сборка,
/// которую он проверяет, не открывая остальные.
pub fn perm_build_access(server_id: &str, build_id: &str) -> String {
    format!("noro.build.{server_id}.{build_id}")
}

#[cfg(test)]
mod tests {
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
        assert!(!permission_matches("noro.server.*", "noro.admin.users"));
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
        assert!(permission_matches("noro.admin.*", "noro.admin.users"));
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
        assert!(any_permission_matches(perms, PERM_ADMIN_WRAPPER));
        assert!(any_permission_matches(perms, &perm_server_join("hitech")));
    }

    #[test]
    fn empty_permission_set_grants_nothing() {
        let empty: [&str; 0] = [];
        assert!(!any_permission_matches(empty, PERM_ADMIN_USERS));
    }

    /// Управление игровой машиной — отдельное право, и админ серверов его не
    /// получает: запись файла плюс рестарт это фактически рут на машине.
    #[test]
    fn server_admin_does_not_imply_wrapper_access() {
        let perms = [PERM_ADMIN_SERVERS];
        assert!(!any_permission_matches(perms, PERM_ADMIN_WRAPPER));
        assert!(any_permission_matches([PERM_ADMIN_ALL], PERM_ADMIN_WRAPPER));
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
}
