//! Система прав на glob-строках и реестр узлов.
//!
//! Узлы заводятся только здесь: `require` принимает константу отсюда, поэтому
//! проверить незарегистрированное право нельзя — не скомпилируется. Реестр
//! отдаётся админке как подсказки, так что новый узел появляется в выдаче
//! ролей сам, без параллельного списка, который разъезжается с кодом.

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
        if target == prefix || target.starts_with(&format!("{prefix}.")) {
            return true;
        }
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

/// Узел прав: имя, группа для админки и человеческое пояснение.
pub struct Node {
    pub name: &'static str,
    pub group: &'static str,
    pub title: &'static str,
}

macro_rules! nodes {
    ($($konst:ident = $name:literal, $group:literal, $title:literal;)*) => {
        $(pub const $konst: &str = $name;)*

        /// Все узлы — для подсказок в редакторе ролей.
        pub const ALL_NODES: &[&Node] = &[$(&Node {
            name: $name, group: $group, title: $title,
        }),*];
    };
}

pub const PERM_SUPERADMIN: &str = "*";

nodes! {
    // --- Панель ---------------------------------------------------------------
    PERM_ADMIN_STATS          = "noro.admin.stats", "perm-group-panel", "perm-node-admin-stats";
    PERM_ADMIN_AGENTS         = "noro.admin.agents", "perm-group-panel", "perm-node-admin-agents";

    // --- Игроки ---------------------------------------------------------------
    PERM_USERS_VIEW           = "noro.admin.users.view", "perm-group-players", "perm-node-users-view";
    PERM_USERS_EDIT           = "noro.admin.users.edit", "perm-group-players", "perm-node-users-edit";
    PERM_USERS_USERNAME       = "noro.admin.users.username", "perm-group-players", "perm-node-users-username";
    PERM_USERS_DELETE         = "noro.admin.users.delete", "perm-group-players", "perm-node-users-delete";
    PERM_USERS_ROLES          = "noro.admin.users.roles", "perm-group-players", "perm-node-users-roles";
    PERM_USERS_PERMISSIONS    = "noro.admin.users.permissions", "perm-group-players", "perm-node-users-permissions";
    PERM_USERS_NOTES_VIEW     = "noro.admin.users.notes.view", "perm-group-players", "perm-node-users-notes-view";
    PERM_USERS_NOTES_WRITE    = "noro.admin.users.notes.write", "perm-group-players", "perm-node-users-notes-write";
    PERM_USERS_NOTES_DELETE   = "noro.admin.users.notes.delete", "perm-group-players", "perm-node-users-notes-delete";
    PERM_USERS_SESSIONS_VIEW  = "noro.admin.users.sessions.view", "perm-group-players", "perm-node-users-sessions-view";
    PERM_USERS_SESSIONS_KILL  = "noro.admin.users.sessions.revoke", "perm-group-players", "perm-node-users-sessions-kill";
    PERM_USERS_JOURNAL        = "noro.admin.users.journal", "perm-group-players", "perm-node-users-journal";
    PERM_USERS_LAUNCHER       = "noro.admin.users.launcher", "perm-group-players", "perm-node-users-launcher";
    PERM_USERS_SKIN           = "noro.admin.users.skin", "perm-group-players", "perm-node-users-skin";
    PERM_USERS_CAPES          = "noro.admin.users.capes", "perm-group-players", "perm-node-users-capes";
    PERM_IMPERSONATE          = "noro.admin.users.impersonate", "perm-group-players", "perm-node-impersonate";

    // --- Модерация ------------------------------------------------------------
    PERM_PUNISH_VIEW          = "noro.mod.punish.view", "perm-group-moderation", "perm-node-punish-view";
    PERM_PUNISH_WARN          = "noro.mod.punish.warn", "perm-group-moderation", "perm-node-punish-warn";
    PERM_PUNISH_MUTE          = "noro.mod.punish.mute", "perm-group-moderation", "perm-node-punish-mute";
    PERM_PUNISH_BAN           = "noro.mod.punish.ban", "perm-group-moderation", "perm-node-punish-ban";
    PERM_PUNISH_SERVER_BAN    = "noro.mod.punish.server_ban", "perm-group-moderation", "perm-node-punish-server-ban";
    PERM_PUNISH_REVOKE        = "noro.mod.punish.revoke", "perm-group-moderation", "perm-node-punish-revoke";
    PERM_PUNISH_BYPASS        = "noro.mod.punish.bypass", "perm-group-moderation", "perm-node-punish-bypass";
    PERM_PUNISH_PERMANENT     = "noro.mod.punish.permanent", "perm-group-moderation", "perm-node-punish-permanent";
    PERM_FREEZE               = "noro.mod.freeze", "perm-group-moderation", "perm-node-freeze";
    PERM_REPORTS_VIEW         = "noro.mod.reports.view", "perm-group-moderation", "perm-node-reports-view";
    PERM_REPORTS_RESOLVE      = "noro.mod.reports.resolve", "perm-group-moderation", "perm-node-reports-resolve";
    PERM_CASES_VIEW           = "noro.mod.cases.view", "perm-group-moderation", "perm-node-cases-view";
    PERM_CASES_CLAIM          = "noro.mod.cases.claim", "perm-group-moderation", "perm-node-cases-claim";
    PERM_CASES_RESOLVE        = "noro.mod.cases.resolve", "perm-group-moderation", "perm-node-cases-resolve";
    PERM_CASES_CHAT           = "noro.mod.cases.chat", "perm-group-moderation", "perm-node-cases-chat";
    PERM_CASES_INVENTORY      = "noro.mod.cases.inventory", "perm-group-moderation", "perm-node-cases-inventory";
    PERM_CASES_WATCH          = "noro.mod.cases.watch", "perm-group-moderation", "perm-node-cases-watch";
    PERM_CASES_CLIENT         = "noro.mod.cases.client", "perm-group-moderation", "perm-node-cases-client";

    // --- Свод правил ----------------------------------------------------------
    PERM_RULES_VIEW           = "noro.admin.rules.view", "perm-group-rules", "perm-node-rules-view";
    PERM_RULES_EDIT           = "noro.admin.rules.edit", "perm-group-rules", "perm-node-rules-edit";
    PERM_RULES_DELETE         = "noro.admin.rules.delete", "perm-group-rules", "perm-node-rules-delete";

    // --- Серверы и сборки -----------------------------------------------------
    PERM_SERVERS_VIEW         = "noro.admin.servers.view", "perm-group-servers", "perm-node-servers-view";
    PERM_SERVERS_EDIT         = "noro.admin.servers.edit", "perm-group-servers", "perm-node-servers-edit";
    PERM_SERVERS_DELETE       = "noro.admin.servers.delete", "perm-group-servers", "perm-node-servers-delete";
    PERM_SERVERS_AGENTS       = "noro.admin.servers.agents", "perm-group-servers", "perm-node-servers-agents";
    PERM_SERVERS_ROLES        = "noro.admin.servers.roles", "perm-group-servers", "perm-node-servers-roles";
    PERM_BUILDS_VIEW          = "noro.admin.builds.view", "perm-group-builds", "perm-node-builds-view";
    PERM_BUILDS_EDIT          = "noro.admin.builds.edit", "perm-group-builds", "perm-node-builds-edit";
    PERM_BUILDS_PUBLISH       = "noro.admin.builds.publish", "perm-group-builds", "perm-node-builds-publish";
    PERM_BUILDS_DELETE        = "noro.admin.builds.delete", "perm-group-builds", "perm-node-builds-delete";
    PERM_BUILDS_IMPORT        = "noro.admin.builds.import", "perm-group-builds", "perm-node-builds-import";
    PERM_MODS_VIEW            = "noro.admin.mods.view", "perm-group-builds", "perm-node-mods-view";
    PERM_MODS_INSTALL         = "noro.admin.mods.install", "perm-group-builds", "perm-node-mods-install";
    PERM_MODS_REMOVE          = "noro.admin.mods.remove", "perm-group-builds", "perm-node-mods-remove";
    PERM_CORES_EDIT           = "noro.admin.cores.edit", "perm-group-builds", "perm-node-cores-edit";

    // --- Игровая машина -------------------------------------------------------
    PERM_WRAPPER_VIEW         = "noro.admin.wrapper.view", "perm-group-machine", "perm-node-wrapper-view";
    PERM_WRAPPER_CONSOLE      = "noro.admin.wrapper.console", "perm-group-machine", "perm-node-wrapper-console";
    PERM_WRAPPER_COMMAND      = "noro.admin.wrapper.command", "perm-group-machine", "perm-node-wrapper-command";
    PERM_WRAPPER_FILES        = "noro.admin.wrapper.files", "perm-group-machine", "perm-node-wrapper-files";
    PERM_WRAPPER_POWER        = "noro.admin.wrapper.power", "perm-group-machine", "perm-node-wrapper-power";
    PERM_WRAPPER_BACKUPS      = "noro.admin.wrapper.backups", "perm-group-machine", "perm-node-wrapper-backups";

    // --- Действия в игре ------------------------------------------------------
    PERM_GAME_KICK            = "noro.admin.game.kick", "perm-group-other", "perm-node-game-kick";
    PERM_GAME_TELL            = "noro.admin.game.tell", "perm-group-other", "perm-node-game-tell";
    PERM_GAME_ANNOUNCE        = "noro.admin.game.announce", "perm-group-other", "perm-node-game-announce";

    // --- Контент --------------------------------------------------------------
    PERM_NEWS_VIEW            = "noro.admin.news.view", "perm-group-content", "perm-node-news-view";
    PERM_NEWS_EDIT            = "noro.admin.news.edit", "perm-group-content", "perm-node-news-edit";
    PERM_NEWS_DELETE          = "noro.admin.news.delete", "perm-group-content", "perm-node-news-delete";
    PERM_TRANSLATIONS_VIEW    = "noro.admin.translations.view", "perm-group-content", "perm-node-translations-view";
    PERM_TRANSLATIONS_EDIT    = "noro.admin.translations.edit", "perm-group-content", "perm-node-translations-edit";
    PERM_CAPES_VIEW           = "noro.admin.capes.view", "perm-group-content", "perm-node-capes-view";
    PERM_CAPES_EDIT           = "noro.admin.capes.edit", "perm-group-content", "perm-node-capes-edit";

    // --- Лаунчер --------------------------------------------------------------
    PERM_LAUNCHER_VIEW        = "noro.admin.launcher.view", "perm-group-launcher", "perm-node-launcher-view";
    PERM_LAUNCHER_PUBLISH     = "noro.admin.launcher.publish", "perm-group-launcher", "perm-node-launcher-publish";
    PERM_LAUNCHER_DEPLOY      = "noro.admin.launcher.deploy", "perm-group-launcher", "perm-node-launcher-deploy";
    PERM_LAUNCHER_CLIENTS     = "noro.admin.launcher.clients", "perm-group-launcher", "perm-node-launcher-clients";
    PERM_LAUNCHER_TOKENS      = "noro.admin.launcher.tokens", "perm-group-launcher", "perm-node-launcher-tokens";

    // --- Целостность и защита -------------------------------------------------
    PERM_INTEGRITY_VIEW       = "noro.admin.integrity.view", "perm-group-security", "perm-node-integrity-view";
    PERM_INTEGRITY_REVIEW     = "noro.admin.integrity.review", "perm-group-security", "perm-node-integrity-review";
    PERM_BLOCKLIST_VIEW       = "noro.admin.blocklist.view", "perm-group-security", "perm-node-blocklist-view";
    PERM_BLOCKLIST_EDIT       = "noro.admin.blocklist.edit", "perm-group-security", "perm-node-blocklist-edit";
    PERM_CHAT_FILTERS_VIEW    = "noro.admin.chat_filters.view", "perm-group-security", "perm-node-chat-filters-view";
    PERM_CHAT_FILTERS_EDIT    = "noro.admin.chat_filters.edit", "perm-group-security", "perm-node-chat-filters-edit";
    PERM_CHAT_FILTERS_DELETE  = "noro.admin.chat_filters.delete", "perm-group-security", "perm-node-chat-filters-delete";

    // --- Поддержка ------------------------------------------------------------
    PERM_SUPPORT_LOGS         = "noro.admin.support.logs", "perm-group-support", "perm-node-support-logs";
    PERM_SUPPORT_REQUEST      = "noro.admin.support.request", "perm-group-support", "perm-node-support-request";
    PERM_SUPPORT_FORCE        = "noro.admin.support.force", "perm-group-support", "perm-node-support-force";
    PERM_SUPPORT_DOWNLOAD     = "noro.admin.support.download", "perm-group-support", "perm-node-support-download";
    PERM_SUPPORT_DELETE       = "noro.admin.support.delete", "perm-group-support", "perm-node-support-delete";

    // --- Модераторские сообщения ----------------------------------------------
    PERM_MODERATION_VIEW      = "noro.admin.moderation.view", "perm-group-moderation", "perm-node-moderation-view";
    PERM_MODERATION_EDIT      = "noro.admin.moderation.edit", "perm-group-moderation", "perm-node-moderation-edit";

    // --- Система --------------------------------------------------------------
    PERM_ROLES_VIEW           = "noro.admin.roles.view", "perm-group-system", "perm-node-roles-view";
    PERM_ROLES_EDIT           = "noro.admin.roles.edit", "perm-group-system", "perm-node-roles-edit";
    PERM_AUDIT                = "noro.admin.audit", "perm-group-system", "perm-node-audit";
    PERM_SETTINGS_VIEW        = "noro.admin.settings.view", "perm-group-system", "perm-node-settings-view";
    PERM_SETTINGS_EDIT        = "noro.admin.settings.edit", "perm-group-system", "perm-node-settings-edit";
    PERM_AUTH_METHODS_VIEW    = "noro.admin.auth_methods.view", "perm-group-system", "perm-node-auth-methods-view";
    PERM_AUTH_METHODS_EDIT    = "noro.admin.auth_methods.edit", "perm-group-system", "perm-node-auth-methods-edit";
    PERM_TOKENS_VIEW          = "noro.admin.tokens.view", "perm-group-system", "perm-node-tokens-view";
    PERM_TOKENS_MANAGE        = "noro.admin.tokens.manage", "perm-group-system", "perm-node-tokens-manage";
    PERM_RESTARTS_VIEW        = "noro.admin.restarts.view", "perm-group-servers", "perm-node-restarts-view";
    PERM_RESTARTS_EDIT        = "noro.admin.restarts.edit", "perm-group-servers", "perm-node-restarts-edit";
    PERM_OAUTH_VIEW           = "noro.admin.oauth.view", "perm-group-system", "perm-node-oauth-view";
    PERM_OAUTH_MANAGE         = "noro.admin.oauth.manage", "perm-group-system", "perm-node-oauth-manage";
    PERM_STORAGE              = "noro.admin.storage", "perm-group-system", "perm-node-storage";
    PERM_BACKUP               = "noro.admin.backup", "perm-group-system", "perm-node-backup";
    PERM_BACKUP_RESTORE       = "noro.admin.backup.restore", "perm-group-system", "perm-node-backup-restore";

    // --- Игрок ----------------------------------------------------------------
    PERM_LAUNCHER_BETA        = "noro.launcher.beta", "perm-group-player", "perm-node-launcher-beta";
}

/// Всё, что открывает админку целиком. Оставлено ради выдачи «полный доступ»
/// одной строкой — точечные узлы для этого пришлось бы перечислять полсотни.
pub const PERM_ADMIN_ALL: &str = "noro.admin.*";

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

/// Право выдавать конкретный вид наказания.
pub fn perm_punish(kind: &str) -> String {
    format!("noro.mod.punish.{kind}")
}

#[cfg(test)]
#[path = "permissions_tests.rs"]
mod tests;
