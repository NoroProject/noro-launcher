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
    // Отдельного права «войти в админку» нет намеренно: панель открывается тем,
    // у кого есть хоть один узел `noro.admin.*`. Иначе выдача точечного права
    // оставляла бы человека перед закрытой дверью.
    PERM_ADMIN_STATS          = "noro.admin.stats",              "Panel", "Dashboard numbers";
    PERM_ADMIN_AGENTS         = "noro.admin.agents",             "Panel", "Agent status list";

    // --- Игроки ---------------------------------------------------------------
    PERM_USERS_VIEW           = "noro.admin.users.view",         "Players", "Player list and profile";
    PERM_USERS_EDIT           = "noro.admin.users.edit",         "Players", "Edit account: links and flags";
    PERM_USERS_USERNAME       = "noro.admin.users.username",     "Players", "Change someone's username";
    PERM_USERS_DELETE         = "noro.admin.users.delete",       "Players", "Delete an account";
    PERM_USERS_ROLES          = "noro.admin.users.roles",        "Players", "Grant and revoke roles";
    PERM_USERS_PERMISSIONS    = "noro.admin.users.permissions",  "Players", "Grant permissions directly";
    PERM_USERS_NOTES_VIEW     = "noro.admin.users.notes.view",   "Players", "Read staff notes";
    PERM_USERS_NOTES_WRITE    = "noro.admin.users.notes.write",  "Players", "Write staff notes";
    PERM_USERS_NOTES_DELETE   = "noro.admin.users.notes.delete", "Players", "Delete staff notes";
    PERM_USERS_SESSIONS_VIEW  = "noro.admin.users.sessions.view", "Players", "See active sessions";
    PERM_USERS_SESSIONS_KILL  = "noro.admin.users.sessions.revoke", "Players", "End someone's sessions";
    PERM_USERS_JOURNAL        = "noro.admin.users.journal",      "Players", "Launch journal and player events";
    PERM_USERS_LAUNCHER       = "noro.admin.users.launcher",     "Players", "Launcher diagnostics and remote actions";
    PERM_USERS_SKIN           = "noro.admin.users.skin",         "Players", "Change someone's skin";
    PERM_USERS_CAPES          = "noro.admin.users.capes",        "Players", "Hand out capes to a player";
    PERM_IMPERSONATE          = "noro.admin.users.impersonate",  "Players", "Act as a player in the launcher";

    // --- Модерация ------------------------------------------------------------
    PERM_PUNISH_VIEW          = "noro.mod.punish.view",          "Moderation", "See punishments";
    PERM_PUNISH_WARN          = "noro.mod.punish.warn",          "Moderation", "Issue warnings";
    PERM_PUNISH_MUTE          = "noro.mod.punish.mute",          "Moderation", "Issue mutes";
    PERM_PUNISH_BAN           = "noro.mod.punish.ban",           "Moderation", "Issue global bans";
    PERM_PUNISH_SERVER_BAN    = "noro.mod.punish.server_ban",    "Moderation", "Ban from one server";
    PERM_PUNISH_REVOKE        = "noro.mod.punish.revoke",        "Moderation", "Lift a punishment";
    // Рамки правила — это и есть ограничение для хелпера: без байпаса он
    // выдаёт ровно то, что записано в своде.
    PERM_PUNISH_BYPASS        = "noro.mod.punish.bypass",        "Moderation", "Ignore the limits set by the rule";
    PERM_PUNISH_PERMANENT     = "noro.mod.punish.permanent",     "Moderation", "Punish forever";
    PERM_FREEZE               = "noro.mod.freeze",               "Moderation", "Freeze/unfreeze a player";
    PERM_REPORTS_VIEW         = "noro.mod.reports.view",         "Moderation", "See player reports";
    PERM_REPORTS_RESOLVE      = "noro.mod.reports.resolve",      "Moderation", "Resolve player reports";

    // --- Свод правил ----------------------------------------------------------
    PERM_RULES_VIEW           = "noro.admin.rules.view",         "Rules", "Open the rulebook editor";
    PERM_RULES_EDIT           = "noro.admin.rules.edit",         "Rules", "Edit rules and sections";
    PERM_RULES_DELETE         = "noro.admin.rules.delete",       "Rules", "Delete rules and sections";

    // --- Серверы и сборки -----------------------------------------------------
    PERM_SERVERS_VIEW         = "noro.admin.servers.view",       "Servers", "See servers and builds";
    PERM_SERVERS_EDIT         = "noro.admin.servers.edit",       "Servers", "Create and edit servers";
    PERM_SERVERS_DELETE       = "noro.admin.servers.delete",     "Servers", "Delete a server";
    PERM_SERVERS_AGENTS       = "noro.admin.servers.agents",     "Servers", "Game servers and agent tokens";
    PERM_SERVERS_ROLES        = "noro.admin.servers.roles",      "Servers", "In-game roles of a server";
    PERM_BUILDS_VIEW          = "noro.admin.builds.view",        "Builds", "See build contents";
    PERM_BUILDS_EDIT          = "noro.admin.builds.edit",        "Builds", "Edit builds and their files";
    PERM_BUILDS_PUBLISH       = "noro.admin.builds.publish",     "Builds", "Publish a build to players";
    // Удаление сборки необратимо, а файлы уходят из стора — отдельный узел.
    PERM_BUILDS_DELETE        = "noro.admin.builds.delete",      "Builds", "Delete a build";
    PERM_BUILDS_IMPORT        = "noro.admin.builds.import",      "Builds", "Import a modpack into a build";
    PERM_MODS_VIEW            = "noro.admin.mods.view",          "Builds", "Browse the mod catalog";
    PERM_MODS_INSTALL         = "noro.admin.mods.install",       "Builds", "Install mods into a build";
    PERM_MODS_REMOVE          = "noro.admin.mods.remove",        "Builds", "Remove mods from a build";
    PERM_CORES_EDIT           = "noro.admin.cores.edit",         "Builds", "Server cores and loaders";

    // --- Игровая машина -------------------------------------------------------
    PERM_WRAPPER_VIEW         = "noro.admin.wrapper.view",       "Machine", "See the game machine state";
    PERM_WRAPPER_CONSOLE      = "noro.admin.wrapper.console",    "Machine", "Read the console";
    // Команда в консоли — это выполнение чего угодно на машине, отдельно от
    // права просто смотреть, что там происходит.
    PERM_WRAPPER_COMMAND      = "noro.admin.wrapper.command",    "Machine", "Send console commands";
    PERM_WRAPPER_FILES        = "noro.admin.wrapper.files",      "Machine", "Read and write server files";
    PERM_WRAPPER_POWER        = "noro.admin.wrapper.power",      "Machine", "Start, stop and restart";
    PERM_WRAPPER_BACKUPS      = "noro.admin.wrapper.backups",    "Machine", "Create and restore backups";

    // --- Действия в игре ------------------------------------------------------
    PERM_GAME_KICK            = "noro.admin.game.kick",          "Game", "Kick player from game";
    PERM_GAME_TELL            = "noro.admin.game.tell",          "Game", "Send private message in game";
    PERM_GAME_ANNOUNCE        = "noro.admin.game.announce",      "Game", "Broadcast announcement in game";

    // --- Контент --------------------------------------------------------------
    PERM_NEWS_VIEW            = "noro.admin.news.view",          "Content", "See news posts";
    PERM_NEWS_EDIT            = "noro.admin.news.edit",          "Content", "Write and publish news";
    PERM_NEWS_DELETE          = "noro.admin.news.delete",        "Content", "Delete news posts";
    PERM_TRANSLATIONS_VIEW    = "noro.admin.translations.view",  "Content", "See interface translations";
    PERM_TRANSLATIONS_EDIT    = "noro.admin.translations.edit",  "Content", "Edit interface translations";
    PERM_CAPES_VIEW           = "noro.admin.capes.view",         "Content", "See capes";
    PERM_CAPES_EDIT           = "noro.admin.capes.edit",         "Content", "Add and remove capes";

    // --- Лаунчер --------------------------------------------------------------
    PERM_LAUNCHER_VIEW        = "noro.admin.launcher.view",      "Launcher", "See launcher releases";
    PERM_LAUNCHER_PUBLISH     = "noro.admin.launcher.publish",   "Launcher", "Publish a launcher release";
    PERM_LAUNCHER_DEPLOY      = "noro.admin.launcher.deploy",    "Launcher", "Roll a release out to players";
    PERM_LAUNCHER_CLIENTS     = "noro.admin.launcher.clients",   "Launcher", "Launcher clients and their state";
    PERM_LAUNCHER_TOKENS      = "noro.admin.launcher.tokens",    "Launcher", "Issue and revoke admin tokens";

    // --- Целостность и защита -------------------------------------------------
    PERM_INTEGRITY_VIEW       = "noro.admin.integrity.view",     "Safety", "See integrity flags";
    PERM_INTEGRITY_REVIEW     = "noro.admin.integrity.review",   "Safety", "Review and close flags";
    PERM_BLOCKLIST_VIEW       = "noro.admin.blocklist.view",     "Safety", "See the blocked files list";
    PERM_BLOCKLIST_EDIT       = "noro.admin.blocklist.edit",     "Safety", "Edit the blocked files list";

    // --- Поддержка ------------------------------------------------------------
    PERM_SUPPORT_LOGS         = "noro.admin.support.logs",       "Support", "Read log bundles";
    PERM_SUPPORT_REQUEST      = "noro.admin.support.request",    "Support", "Ask a player for logs";
    // Отдельно от чтения: нужен ровно там, где согласие бессмысленно — иначе
    // единственный, чьи логи никогда не придут, это тот, ради кого всё затевалось.
    PERM_SUPPORT_FORCE        = "noro.admin.support.force",      "Support", "Collect logs without consent";
    // В бандле лежит содержимое чужого компьютера: скачивание отдельно от
    // просмотра списка.
    PERM_SUPPORT_DOWNLOAD     = "noro.admin.support.download",   "Support", "Download a log bundle";
    PERM_SUPPORT_DELETE       = "noro.admin.support.delete",     "Support", "Delete log bundles";

    // --- Система --------------------------------------------------------------
    PERM_ROLES_VIEW           = "noro.admin.roles.view",         "System", "See roles";
    PERM_ROLES_EDIT           = "noro.admin.roles.edit",         "System", "Create and edit roles";
    PERM_AUDIT                = "noro.admin.audit",              "System", "Read the admin journal";
    PERM_SETTINGS_VIEW        = "noro.admin.settings.view",      "System", "See instance settings";
    PERM_SETTINGS_EDIT        = "noro.admin.settings.edit",      "System", "Change instance settings";
    // Операция удаляет файлы с диска, и восстановить их можно только
    // перезаливкой сборки.
    PERM_STORAGE              = "noro.admin.storage",            "System", "Clean unused storage objects";
    // В дампе лежат все пользователи, их привязки и токены сессий — это самый
    // чувствительный объект в системе.
    PERM_BACKUP               = "noro.admin.backup",             "System", "Download a database dump";

    // --- Игрок ----------------------------------------------------------------
    PERM_LAUNCHER_BETA        = "noro.launcher.beta",            "Player", "Launcher beta channel";
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
