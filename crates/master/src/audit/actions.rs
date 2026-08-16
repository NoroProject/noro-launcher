//! Реестр событий аудита.
//!
//! Список нужен интерфейсу: без него фильтр — это поле, куда надо угадать
//! строку. Держится здесь, рядом с кодом, который эти события пишет, — иначе
//! он разойдётся с действительностью в первый же месяц.

/// Событие: имя, группа и человеческое описание.
pub struct Action {
    pub name: &'static str,
    pub group: &'static str,
    pub title: &'static str,
}

macro_rules! actions {
    ($($name:literal, $group:literal, $title:literal;)*) => {
        pub const ALL: &[Action] = &[
            $(Action { name: $name, group: $group, title: $title }),*
        ];
    };
}

actions! {
    // Вход и сессии
    "auth.login",                   "Auth",       "Вход в аккаунт";
    "auth.recovery_code",           "Auth",       "Вход по recovery-коду";
    "auth.recovery_codes.reissue",  "Auth",       "Перевыпуск recovery-кодов";
    "user.session.revoke",          "Auth",       "Завершена сессия";
    "user.sessions.revoke",         "Auth",       "Завершены все сессии";

    // Игра
    "game.start",                   "Game",       "Запуск сборки";
    "game.stop",                    "Game",       "Выход из игры";
    "integrity.findings",           "Game",       "Расхождения при сверке";
    "integrity.review",             "Game",       "Флаг разобран";

    // Игроки
    "user.ban",                     "Users",      "Бан";
    "user.unban",                   "Users",      "Разбан";
    "user.role.add",                "Users",      "Выдана роль";
    "user.role.remove",             "Users",      "Снята роль";
    "user.permission.add",          "Users",      "Выдано право";
    "user.permission.remove",       "Users",      "Снято право";
    "punishment.ban",               "Users",      "Наказание: бан";
    "punishment.warn",              "Users",      "Наказание: предупреждение";
    "punishment.server_ban",        "Users",      "Наказание: доступ к серверу";
    "punishment.revoke",            "Users",      "Наказание снято";

    // Роли
    "role.create",                  "Roles",      "Создана роль";
    "role.update",                  "Roles",      "Изменена роль";
    "role.delete",                  "Roles",      "Удалена роль";
    "role.permission.add",          "Roles",      "Роли выдано право";
    "role.permission.remove",       "Roles",      "У роли снято право";

    // Поддержка
    "support.logs.request",         "Support",    "Запрошены логи";
    "support.logs.force",           "Support",    "Логи собраны без спроса";
    "support.bundle.upload",        "Support",    "Получен бандл логов";
    "user.remote_action",           "Support",    "Удалённое действие";

    // Impersonation
    "impersonate.request",          "Impersonate", "Запрошен вход в аккаунт";
    "impersonate.start",            "Impersonate", "Начата сессия от имени игрока";
    "impersonate.action",           "Impersonate", "Действие под чужим аккаунтом";

    // Контент и система
    "build.publish",                "Content",    "Сборка опубликована";
    "build.unpublish",              "Content",    "Сборка снята с публикации";
    "server.update",                "Content",    "Изменён сервер";
    "server.delete",                "Content",    "Удалён сервер";
    "launcher.deploy",              "System",     "Выкачена версия лаунчера";
    "settings.update",              "System",     "Изменены настройки";
    "storage.gc",                   "System",     "Уборка хранилища";
    "admin_token.create",           "System",     "Создан admin-токен";
    "admin_token.delete",           "System",     "Удалён admin-токен";
    "blocklist.add",                "System",     "Добавлено правило блокировки";
    "blocklist.remove",             "System",     "Удалено правило блокировки";
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
