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
    // Умный неявный доступ на просмотр: если у пользователя есть право на действие в подсистеме
    // (например, `noro.admin.users.edit` или `noro.mod.punish.ban`), он автоматически имеет право
    // видеть список в этой подсистеме (`.view`), но НЕ дочерние детали вроде `.roles` или `.permissions`.
    if target.ends_with(".view") {
        let base = target.strip_suffix(".view").unwrap();
        if pattern.starts_with(&format!("{base}.")) {
            return true;
        }
        if target == "noro.admin.users.view"
            && (pattern.starts_with("noro.mod.punish.") || pattern.starts_with("noro.admin.users."))
        {
            return true;
        }
        if target == "noro.admin.servers.view" && pattern.starts_with("noro.admin.builds.") {
            return true;
        }
        if target == "noro.mod.cases.view" && pattern.starts_with("noro.mod.cases.") {
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
    PERM_ADMIN_STATS          = "noro.admin.stats",              "Панель", "Метрики и статистика панели";
    PERM_ADMIN_AGENTS         = "noro.admin.agents",             "Панель", "Список статусов агентов";

    // --- Игроки ---------------------------------------------------------------
    PERM_USERS_VIEW           = "noro.admin.users.view",         "Игроки", "Просмотр списка игроков и профиля";
    PERM_USERS_EDIT           = "noro.admin.users.edit",         "Игроки", "Редактирование аккаунта и привязок";
    PERM_USERS_USERNAME       = "noro.admin.users.username",     "Игроки", "Смена игрового ника";
    PERM_USERS_DELETE         = "noro.admin.users.delete",       "Игроки", "Удаление аккаунтов игроков";
    PERM_USERS_ROLES          = "noro.admin.users.roles",        "Игроки", "Назначение и снятие ролей";
    PERM_USERS_PERMISSIONS    = "noro.admin.users.permissions",  "Игроки", "Прямая выдача персональных прав";
    PERM_USERS_NOTES_VIEW     = "noro.admin.users.notes.view",   "Игроки", "Просмотр заметок персонала";
    PERM_USERS_NOTES_WRITE    = "noro.admin.users.notes.write",  "Игроки", "Создание заметок персонала";
    PERM_USERS_NOTES_DELETE   = "noro.admin.users.notes.delete", "Игроки", "Удаление заметок персонала";
    PERM_USERS_SESSIONS_VIEW  = "noro.admin.users.sessions.view", "Игроки", "Просмотр активных сессий";
    PERM_USERS_SESSIONS_KILL  = "noro.admin.users.sessions.revoke", "Игроки", "Завершение сессий игроков";
    PERM_USERS_JOURNAL        = "noro.admin.users.journal",      "Игроки", "Журнал запусков и событий";
    PERM_USERS_LAUNCHER       = "noro.admin.users.launcher",     "Игроки", "Диагностика и удаленные действия";
    PERM_USERS_SKIN           = "noro.admin.users.skin",         "Игроки", "Смена скина игроку";
    PERM_USERS_CAPES          = "noro.admin.users.capes",        "Игроки", "Выдача плащей игроку";
    PERM_IMPERSONATE          = "noro.admin.users.impersonate",  "Игроки", "Вход под аккаунтом игрока";

    // --- Модерация ------------------------------------------------------------
    PERM_PUNISH_VIEW          = "noro.mod.punish.view",          "Модерация", "Просмотр истории наказаний";
    PERM_PUNISH_WARN          = "noro.mod.punish.warn",          "Модерация", "Выдача предупреждений (варнов)";
    PERM_PUNISH_MUTE          = "noro.mod.punish.mute",          "Модерация", "Выдача блокировок чата (мутов)";
    PERM_PUNISH_BAN           = "noro.mod.punish.ban",           "Модерация", "Выдача глобальных банов";
    PERM_PUNISH_SERVER_BAN    = "noro.mod.punish.server_ban",    "Модерация", "Бан на конкретном сервере";
    PERM_PUNISH_REVOKE        = "noro.mod.punish.revoke",        "Модерация", "Снятие наказаний";
    PERM_PUNISH_BYPASS        = "noro.mod.punish.bypass",        "Модерация", "Обход ограничений правил";
    PERM_PUNISH_PERMANENT     = "noro.mod.punish.permanent",     "Модерация", "Выдача перманентных наказаний";
    PERM_FREEZE               = "noro.mod.freeze",               "Модерация", "Заморозка игроков";
    PERM_REPORTS_VIEW         = "noro.mod.reports.view",         "Модерация", "Просмотр жалоб игроков";
    PERM_REPORTS_RESOLVE      = "noro.mod.reports.resolve",      "Модерация", "Обработка жалоб игроков";
    PERM_CASES_VIEW           = "noro.mod.cases.view",           "Модерация", "Просмотр очереди и карточек дел";
    PERM_CASES_CLAIM          = "noro.mod.cases.claim",          "Модерация", "Взятие дела в работу";
    PERM_CASES_RESOLVE        = "noro.mod.cases.resolve",        "Модерация", "Закрытие дела с вердиктом";
    PERM_CASES_CHAT           = "noro.mod.cases.chat",           "Модерация", "Просмотр среза чата в деле";
    PERM_CASES_INVENTORY      = "noro.mod.cases.inventory",      "Модерация", "Просмотр инвентаря игрока";
    PERM_CASES_WATCH          = "noro.mod.cases.watch",          "Модерация", "Слежка за игроком";
    PERM_CASES_CLIENT         = "noro.mod.cases.client",         "Модерация", "Проверка лаунчера игрока";

    // --- Свод правил ----------------------------------------------------------
    PERM_RULES_VIEW           = "noro.admin.rules.view",         "Правила", "Просмотр свода правил";
    PERM_RULES_EDIT           = "noro.admin.rules.edit",         "Правила", "Создание и редактирование правил";
    PERM_RULES_DELETE         = "noro.admin.rules.delete",       "Правила", "Удаление правил и разделов";

    // --- Серверы и сборки -----------------------------------------------------
    PERM_SERVERS_VIEW         = "noro.admin.servers.view",       "Серверы", "Просмотр серверов и сборок";
    PERM_SERVERS_EDIT         = "noro.admin.servers.edit",       "Серверы", "Создание и редактирование серверов";
    PERM_SERVERS_DELETE       = "noro.admin.servers.delete",     "Серверы", "Удаление серверов";
    PERM_SERVERS_AGENTS       = "noro.admin.servers.agents",     "Серверы", "Игровые серверы и токены агентов";
    PERM_SERVERS_ROLES        = "noro.admin.servers.roles",      "Серверы", "Игровые роли сервера";
    PERM_BUILDS_VIEW          = "noro.admin.builds.view",        "Сборки", "Просмотр содержимого сборок";
    PERM_BUILDS_EDIT          = "noro.admin.builds.edit",        "Сборки", "Редактирование сборок и файлов";
    PERM_BUILDS_PUBLISH       = "noro.admin.builds.publish",     "Сборки", "Публикация сборок игрокам";
    PERM_BUILDS_DELETE        = "noro.admin.builds.delete",      "Сборки", "Удаление сборок";
    PERM_BUILDS_IMPORT        = "noro.admin.builds.import",      "Сборки", "Импорт сборки модов";
    PERM_MODS_VIEW            = "noro.admin.mods.view",          "Сборки", "Просмотр каталога модов";
    PERM_MODS_INSTALL         = "noro.admin.mods.install",       "Сборки", "Установка модов в сборку";
    PERM_MODS_REMOVE          = "noro.admin.mods.remove",        "Сборки", "Удаление модов из сборки";
    PERM_CORES_EDIT           = "noro.admin.cores.edit",         "Сборки", "Ядра серверов и загрузчики";

    // --- Игровая машина -------------------------------------------------------
    PERM_WRAPPER_VIEW         = "noro.admin.wrapper.view",       "Машина", "Просмотр состояния игровой машины";
    PERM_WRAPPER_CONSOLE      = "noro.admin.wrapper.console",    "Машина", "Чтение консоли сервера";
    PERM_WRAPPER_COMMAND      = "noro.admin.wrapper.command",    "Машина", "Отправка команд в консоль";
    PERM_WRAPPER_FILES        = "noro.admin.wrapper.files",      "Машина", "Чтение и запись файлов сервера";
    PERM_WRAPPER_POWER        = "noro.admin.wrapper.power",      "Машина", "Управление питанием и перезапуск";
    PERM_WRAPPER_BACKUPS      = "noro.admin.wrapper.backups",    "Машина", "Создание и восстановление бэкапов";

    // --- Действия в игре ------------------------------------------------------
    PERM_GAME_KICK            = "noro.admin.game.kick",          "Действия в игре", "Кик игрока из игры";
    PERM_GAME_TELL            = "noro.admin.game.tell",          "Действия в игре", "Отправка ЛС игроку в игре";
    PERM_GAME_ANNOUNCE        = "noro.admin.game.announce",      "Действия в игре", "Объявление в игре";

    // --- Контент --------------------------------------------------------------
    PERM_NEWS_VIEW            = "noro.admin.news.view",          "Контент", "Просмотр новостей";
    PERM_NEWS_EDIT            = "noro.admin.news.edit",          "Контент", "Создание и публикация новостей";
    PERM_NEWS_DELETE          = "noro.admin.news.delete",        "Контент", "Удаление новостей";
    PERM_TRANSLATIONS_VIEW    = "noro.admin.translations.view",  "Контент", "Просмотр переводов интерфейса";
    PERM_TRANSLATIONS_EDIT    = "noro.admin.translations.edit",  "Контент", "Редактирование переводов интерфейса";
    PERM_CAPES_VIEW           = "noro.admin.capes.view",         "Контент", "Просмотр плащей";
    PERM_CAPES_EDIT           = "noro.admin.capes.edit",         "Контент", "Загрузка и удаление плащей";

    // --- Лаунчер --------------------------------------------------------------
    PERM_LAUNCHER_VIEW        = "noro.admin.launcher.view",      "Лаунчер", "Просмотр релизов лаунчера";
    PERM_LAUNCHER_PUBLISH     = "noro.admin.launcher.publish",   "Лаунчер", "Публикация релиза лаунчера";
    PERM_LAUNCHER_DEPLOY      = "noro.admin.launcher.deploy",    "Лаунчер", "Выкатка релизов игрокам";
    PERM_LAUNCHER_CLIENTS     = "noro.admin.launcher.clients",   "Лаунчер", "Клиенты лаунчера и их статус";
    PERM_LAUNCHER_TOKENS      = "noro.admin.launcher.tokens",    "Лаунчер", "Выдача API-токенов лаунчера";

    // --- Целостность и защита -------------------------------------------------
    PERM_INTEGRITY_VIEW       = "noro.admin.integrity.view",     "Защита", "Просмотр флагов целостности";
    PERM_INTEGRITY_REVIEW     = "noro.admin.integrity.review",   "Защита", "Разбор флагов целостности";
    PERM_BLOCKLIST_VIEW       = "noro.admin.blocklist.view",     "Защита", "Просмотр черного списка файлов";
    PERM_BLOCKLIST_EDIT       = "noro.admin.blocklist.edit",     "Защита", "Редактирование черного списка";
    PERM_CHAT_FILTERS_VIEW    = "noro.admin.chat_filters.view",  "Защита", "Просмотр фильтров чата";
    PERM_CHAT_FILTERS_EDIT    = "noro.admin.chat_filters.edit",  "Защита", "Настройка автомодерации и фильтров";
    PERM_CHAT_FILTERS_DELETE  = "noro.admin.chat_filters.delete","Защита", "Удаление фильтров чата";

    // --- Поддержка ------------------------------------------------------------
    PERM_SUPPORT_LOGS         = "noro.admin.support.logs",       "Поддержка", "Просмотр бандлов логов";
    PERM_SUPPORT_REQUEST      = "noro.admin.support.request",    "Поддержка", "Запрос логов у игрока";
    PERM_SUPPORT_FORCE        = "noro.admin.support.force",      "Поддержка", "Принудительный сбор логов";
    PERM_SUPPORT_DOWNLOAD     = "noro.admin.support.download",   "Поддержка", "Скачивание бандлов логов";
    PERM_SUPPORT_DELETE       = "noro.admin.support.delete",     "Поддержка", "Удаление логов поддержки";

    // --- Модераторские сообщения ----------------------------------------------
    PERM_MODERATION_VIEW      = "noro.admin.moderation.view",    "Модерация", "Просмотр модераторских шаблонов";
    PERM_MODERATION_EDIT      = "noro.admin.moderation.edit",    "Модерация", "Редактирование модераторских шаблонов";

    // --- Система --------------------------------------------------------------
    PERM_ROLES_VIEW           = "noro.admin.roles.view",         "Система", "Просмотр ролей";
    PERM_ROLES_EDIT           = "noro.admin.roles.edit",         "Система", "Создание и редактирование ролей";
    PERM_AUDIT                = "noro.admin.audit",              "Система", "Просмотр журнала аудита";
    PERM_SETTINGS_VIEW        = "noro.admin.settings.view",      "Система", "Просмотр настроек инстанса";
    PERM_SETTINGS_EDIT        = "noro.admin.settings.edit",      "Система", "Изменение настроек инстанса";
    PERM_AUTH_METHODS_VIEW    = "noro.admin.auth_methods.view",  "Система", "Просмотр способов входа";
    PERM_AUTH_METHODS_EDIT    = "noro.admin.auth_methods.edit",  "Система", "Настройка способов входа";
    PERM_TOKENS_VIEW          = "noro.admin.tokens.view",        "Система", "Просмотр API-токенов панели";
    PERM_TOKENS_MANAGE        = "noro.admin.tokens.manage",      "Система", "Создание и отзыв API-токенов панели";
    PERM_RESTARTS_VIEW        = "noro.admin.restarts.view",      "Серверы", "Просмотр расписания перезапусков";
    PERM_RESTARTS_EDIT        = "noro.admin.restarts.edit",      "Серверы", "Управление расписанием перезапусков";
    PERM_OAUTH_VIEW           = "noro.admin.oauth.view",         "Система", "Просмотр OAuth2-приложений";
    PERM_OAUTH_MANAGE         = "noro.admin.oauth.manage",       "Система", "Модерация OAuth2-приложений";
    PERM_STORAGE              = "noro.admin.storage",            "Система", "Очистка неиспользуемых объектов стора";
    PERM_BACKUP               = "noro.admin.backup",             "Система", "Скачивание дампа базы данных";
    PERM_BACKUP_RESTORE       = "noro.admin.backup.restore",     "Система", "Восстановление мастера из архива";

    // --- Игрок ----------------------------------------------------------------
    PERM_LAUNCHER_BETA        = "noro.launcher.beta",            "Игрок", "Бета-канал лаунчера";
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
