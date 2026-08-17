//! Реестр событий аудита — единственное место, где заводится событие.
//!
//! `record` принимает не строку, а константу отсюда, поэтому написать в журнал
//! незарегистрированное событие нельзя: не скомпилируется. Раньше список был
//! параллельным — его набирали глазами по вызовам, и он расходился с кодом
//! ровно до первого забытого события.
//!
//! Заводя новое событие: строка сюда, дальше компилятор доведёт до места.

/// Событие: имя в журнале, группа и человеческое название для фильтра.
pub struct Action {
    pub name: &'static str,
    pub group: &'static str,
    pub title: &'static str,
}

macro_rules! actions {
    ($($konst:ident = $name:literal, $group:literal, $title:literal;)*) => {
        $(pub const $konst: &Action = &Action {
            name: $name, group: $group, title: $title,
        };)*

        /// Всё, что бывает, — для выпадающего списка в фильтре.
        pub const ALL: &[&Action] = &[$($konst),*];
    };
}

actions! {
    // Вход и сессии
    AUTH_LOGIN            = "auth.login",                  "Auth",  "Signed in";
    AUTH_RECOVERY_CODE    = "auth.recovery_code",          "Auth",  "Signed in with a recovery code";
    AUTH_RECOVERY_REISSUE = "auth.recovery_codes.reissue", "Auth",  "Recovery codes reissued";
    SESSION_REVOKE        = "user.session.revoke",         "Auth",  "Session ended";
    SESSIONS_REVOKE       = "user.sessions.revoke",        "Auth",  "All sessions ended";

    // Игра
    GAME_START            = "game.start",                  "Game",  "Build launched";
    GAME_STOP             = "game.stop",                   "Game",  "Game closed";
    INTEGRITY_FINDINGS    = "integrity.findings",          "Game",  "Integrity mismatch";
    INTEGRITY_REVIEW      = "integrity.review",            "Game",  "Flag reviewed";

    // Игроки
    USER_BAN              = "user.ban",                    "Users", "Banned";
    USER_UNBAN            = "user.unban",                  "Users", "Unbanned";
    USER_ROLE_ADD         = "user.role.add",               "Users", "Role granted";
    USER_ROLE_REMOVE      = "user.role.remove",            "Users", "Role revoked";
    USER_PERM_ADD         = "user.permission.add",         "Users", "Permission granted";
    USER_PERM_REMOVE      = "user.permission.remove",      "Users", "Permission revoked";
    PUNISHMENT_BAN        = "punishment.ban",              "Users", "Punishment: ban";
    PUNISHMENT_WARN       = "punishment.warn",             "Users", "Punishment: warning";
    PUNISHMENT_SERVER_BAN = "punishment.server_ban",       "Users", "Punishment: server access";
    PUNISHMENT_MUTE       = "punishment.mute",             "Users", "Punishment: mute";
    PUNISHMENT_REVOKE     = "punishment.revoke",           "Users", "Punishment lifted";

    // Роли
    ROLE_CREATE           = "role.create",                 "Roles", "Role created";
    ROLE_UPDATE           = "role.update",                 "Roles", "Role edited";
    ROLE_DELETE           = "role.delete",                 "Roles", "Role deleted";
    ROLE_PERM_ADD         = "role.permission.add",         "Roles", "Role gained a permission";
    ROLE_PERM_REMOVE      = "role.permission.remove",      "Roles", "Role lost a permission";

    // Поддержка
    LOGS_REQUEST          = "support.logs.request",        "Support", "Logs requested";
    LOGS_FORCE            = "support.logs.force",          "Support", "Logs collected without asking";
    LOGS_CANCEL           = "support.logs.cancel",         "Support", "Log request cancelled";
    BUNDLE_UPLOAD         = "support.bundle.upload",       "Support", "Log bundle received";
    REMOTE_ACTION         = "user.remote_action",          "Support", "Remote action";

    // Impersonation
    IMPERSONATE_REQUEST   = "impersonate.request",         "Impersonate", "Impersonation requested";
    IMPERSONATE_START     = "impersonate.start",           "Impersonate", "Impersonation started";
    IMPERSONATE_ACTION    = "impersonate.action",          "Impersonate", "Action while impersonating";

    // Контент и система
    BUILD_PUBLISH         = "build.publish",               "Content", "Build published";
    BUILD_UNPUBLISH       = "build.unpublish",             "Content", "Build unpublished";
    SERVER_UPDATE         = "server.update",               "Content", "Server edited";
    SERVER_DELETE         = "server.delete",               "Content", "Server deleted";
    LAUNCHER_DEPLOY       = "launcher.deploy",             "System", "Launcher version deployed";
    SETTINGS_UPDATE       = "settings.update",             "System", "Settings changed";
    STORAGE_GC            = "storage.gc",                  "System", "Storage cleaned up";
    ADMIN_TOKEN_CREATE    = "admin_token.create",          "System", "Admin token created";
    ADMIN_TOKEN_DELETE    = "admin_token.delete",          "System", "Admin token deleted";
    BLOCKLIST_ADD         = "blocklist.add",               "System", "Blocklist rule added";
    BLOCKLIST_REMOVE      = "blocklist.remove",            "System", "Blocklist rule removed";
}

/// Группы в порядке показа.
pub const GROUPS: &[&str] = &[
    "Auth",
    "Game",
    "Users",
    "Roles",
    "Support",
    "Impersonate",
    "Content",
    "System",
];

/// Наказание по виду — единственное место, где событие выбирается на ходу.
/// Вид проверен обработчиком до этого вызова, так что остаток — предупреждение.
pub fn punishment(kind: &str) -> &'static Action {
    match kind {
        "ban" => PUNISHMENT_BAN,
        "server_ban" => PUNISHMENT_SERVER_BAN,
        "mute" => PUNISHMENT_MUTE,
        _ => PUNISHMENT_WARN,
    }
}
