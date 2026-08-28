//! Default texts for punishment templates.
//!
//! These are player-facing strings; keep them in their own language. The
//! Russian set is the original rather than a translation of the English one —
//! the migration that moves the old flat value points at it.
//!
//! Three greys throughout: value, label, secondary. No caps, no `[!]`, no
//! exclamation marks, on purpose.

use super::moderation_messages::ModerationMessages;

/// The value the player is reading the screen for.
const FG: &str = "<#e6e6e6>";
/// Field label.
const DIM: &str = "<#8b8b8b>";
/// Secondary: case number, link.
const MUTED: &str = "<#5c5c5c>";

impl ModerationMessages {
    pub fn default_ru() -> Self {
        Self {
            ban_permanent: format!("{FG}<bold>Блокировка сети\n\n{DIM}Причина: {FG}{{reason}}\n{DIM}Правило: {FG}{{rule_link}} {DIM}{{rule_title}}\n{DIM}Срок: {FG}бессрочно\n{DIM}Модератор: {FG}{{actor}}\n{DIM}Дело: {MUTED}{{id}}\n\n{MUTED}Обжалование: https://example.com/support"),
            ban_temporary: format!("{FG}<bold>Блокировка сети\n\n{DIM}Причина: {FG}{{reason}}\n{DIM}Правило: {FG}{{rule_link}} {DIM}{{rule_title}}\n{DIM}Срок: {FG}{{duration}} {DIM}— до {FG}{{expires}}\n{DIM}Модератор: {FG}{{actor}}\n{DIM}Дело: {MUTED}{{id}}\n\n{MUTED}Обжалование: https://example.com/support"),
            server_ban_permanent: format!("{FG}<bold>Блокировка сервера\n\n{DIM}Причина: {FG}{{reason}}\n{DIM}Правило: {FG}{{rule_link}} {DIM}{{rule_title}}\n{DIM}Срок: {FG}бессрочно\n{DIM}Модератор: {FG}{{actor}}\n{DIM}Дело: {MUTED}{{id}}\n\n{MUTED}Остальные серверы сети остаются доступны."),
            server_ban_temporary: format!("{FG}<bold>Блокировка сервера\n\n{DIM}Причина: {FG}{{reason}}\n{DIM}Правило: {FG}{{rule_link}} {DIM}{{rule_title}}\n{DIM}Срок: {FG}{{duration}} {DIM}— до {FG}{{expires}}\n{DIM}Модератор: {FG}{{actor}}\n{DIM}Дело: {MUTED}{{id}}\n\n{MUTED}Остальные серверы сети остаются доступны."),
            mute_permanent: format!("{DIM}Сообщение не отправлено: чат для вас закрыт. Причина: {FG}{{reason}}"),
            mute_temporary: format!("{DIM}Сообщение не отправлено: чат закрыт ещё {FG}{{duration}}{DIM}. Причина: {FG}{{reason}}"),
            mute_actionbar_permanent: format!("{DIM}Чат закрыт · {FG}{{reason}}"),
            mute_actionbar_temporary: format!("{DIM}Чат закрыт ещё {FG}{{duration}}"),
            warn_actionbar: format!("{DIM}Предупреждение · {FG}{{reason}}"),
            warn_notice: format!("{DIM}Предупреждение от {FG}{{actor}}{DIM}: {FG}{{reason}} {MUTED}{{rule_link}}"),
            broadcast: format!("{MUTED}{{player}} — {{kind}}: {{reason}}"),
            actor_receipt: format!("{DIM}Выдано: {FG}{{player}} {DIM}— {{kind}}, {{reason}}"),
            reason_by_rule: "п.{rule} — {rule_title}".into(),
            no_account: format!("{FG}<bold>Нужен аккаунт сети\n\n{DIM}Этот ник не привязан к аккаунту.\n{DIM}Зайдите через лаунчер — он создаст аккаунт и вернёт вас сюда.\n\n{MUTED}https://example.com"),
            no_access: format!("{FG}<bold>Доступ закрыт\n\n{DIM}Ваш аккаунт не допущен к этой сборке.\n{DIM}Если это ошибка, напишите в поддержку.\n\n{MUTED}https://example.com/support"),
            maintenance: format!("{FG}<bold>Технические работы\n\n{DIM}Сервер закрыт на обслуживание. Зайдите позже."),
        }
    }

    pub fn default_en() -> Self {
        Self {
            ban_permanent: format!("{FG}<bold>Network ban\n\n{DIM}Reason: {FG}{{reason}}\n{DIM}Rule: {FG}{{rule_link}} {DIM}{{rule_title}}\n{DIM}Term: {FG}permanent\n{DIM}Moderator: {FG}{{actor}}\n{DIM}Case: {MUTED}{{id}}\n\n{MUTED}Appeal: https://example.com/support"),
            ban_temporary: format!("{FG}<bold>Network ban\n\n{DIM}Reason: {FG}{{reason}}\n{DIM}Rule: {FG}{{rule_link}} {DIM}{{rule_title}}\n{DIM}Term: {FG}{{duration}} {DIM}— until {FG}{{expires}}\n{DIM}Moderator: {FG}{{actor}}\n{DIM}Case: {MUTED}{{id}}\n\n{MUTED}Appeal: https://example.com/support"),
            server_ban_permanent: format!("{FG}<bold>Server ban\n\n{DIM}Reason: {FG}{{reason}}\n{DIM}Rule: {FG}{{rule_link}} {DIM}{{rule_title}}\n{DIM}Term: {FG}permanent\n{DIM}Moderator: {FG}{{actor}}\n{DIM}Case: {MUTED}{{id}}\n\n{MUTED}Other network servers remain available."),
            server_ban_temporary: format!("{FG}<bold>Server ban\n\n{DIM}Reason: {FG}{{reason}}\n{DIM}Rule: {FG}{{rule_link}} {DIM}{{rule_title}}\n{DIM}Term: {FG}{{duration}} {DIM}— until {FG}{{expires}}\n{DIM}Moderator: {FG}{{actor}}\n{DIM}Case: {MUTED}{{id}}\n\n{MUTED}Other network servers remain available."),
            mute_permanent: format!("{DIM}Message not sent: your chat is closed. Reason: {FG}{{reason}}"),
            mute_temporary: format!("{DIM}Message not sent: chat closed for another {FG}{{duration}}{DIM}. Reason: {FG}{{reason}}"),
            mute_actionbar_permanent: format!("{DIM}Chat closed · {FG}{{reason}}"),
            mute_actionbar_temporary: format!("{DIM}Chat closed for another {FG}{{duration}}"),
            warn_actionbar: format!("{DIM}Warning · {FG}{{reason}}"),
            warn_notice: format!("{DIM}Warning from {FG}{{actor}}{DIM}: {FG}{{reason}} {MUTED}{{rule_link}}"),
            broadcast: format!("{MUTED}{{player}} — {{kind}}: {{reason}}"),
            actor_receipt: format!("{DIM}Issued: {FG}{{player}} {DIM}— {{kind}}, {{reason}}"),
            reason_by_rule: "s.{rule} — {rule_title}".into(),
            no_account: format!("{FG}<bold>Network account required\n\n{DIM}This username is not linked to an account.\n{DIM}Sign in through the launcher — it will create one and bring you back.\n\n{MUTED}https://example.com"),
            no_access: format!("{FG}<bold>Access closed\n\n{DIM}Your account is not admitted to this build.\n{DIM}If this is a mistake, contact support.\n\n{MUTED}https://example.com/support"),
            maintenance: format!("{FG}<bold>Maintenance\n\n{DIM}The server is closed for maintenance. Try again later."),
        }
    }
}
