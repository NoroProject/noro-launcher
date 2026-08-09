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
}
