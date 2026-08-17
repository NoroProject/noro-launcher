//! Шаблоны сообщений о наказаниях: экран кика, отказ в чате, текст варна.
//!
//! Живут в `instance_settings` одним JSON, а не полями конфига: их правят из
//! админки на живом сервере, а перезапуск мастера ради формулировки бана —
//! слишком дорогая цена за запятую.
//!
//! Подстановки: `{player}`, `{reason}`, `{duration}`, `{expires}`, `{actor}`,
//! `{rule}`, `{id}`. Рисует их агент — мастер отдаёт шаблоны как есть.

use serde::{Deserialize, Serialize};

/// Ключ в `instance_settings`.
pub const SETTINGS_KEY: &str = "moderation_messages";

/// Тексты по умолчанию. Английские: интерфейс игры и лаунчера у нас английский.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ModerationMessages {
    /// Экран отключения при вечном бане.
    pub ban_permanent: String,
    /// Экран отключения при бане со сроком.
    pub ban_temporary: String,
    pub server_ban_permanent: String,
    pub server_ban_temporary: String,
    /// Ответ на попытку написать в чат.
    pub mute_permanent: String,
    pub mute_temporary: String,
    /// Показывается наказанному, когда он в сети.
    pub warn_notice: String,
    /// Показывается всем на сервере. Пусто — не объявлять.
    pub broadcast: String,
    /// Подтверждение тому, кто выдал наказание.
    pub actor_receipt: String,
}

impl Default for ModerationMessages {
    fn default() -> Self {
        Self {
            ban_permanent: "#f87171&lДОСТУП В СЕТЬ ЗАБЛОКИРОВАН #94a3b8(Навсегда)\n\n#94a3b8Причина: #f8fafc{reason}\n#94a3b8Модератор: #f8fafc{actor}\n#94a3b8Правило: #f8fafc{rule}\n#94a3b8Дело: #64748b#{id}\n\n#64748bАпелляция: https://noro.dalynkaa.dev/support".into(),
            ban_temporary: "#f87171&lДОСТУП В СЕТЬ ОГРАНИЧЕН\n\n#94a3b8Причина: #f8fafc{reason}\n#94a3b8Модератор: #f8fafc{actor}\n#94a3b8Окончание: #fbbf24{expires} #94a3b8(через #fbbf24{duration}#94a3b8)\n#94a3b8Правило: #f8fafc{rule}\n#94a3b8Дело: #64748b#{id}\n\n#64748bАпелляция: https://noro.dalynkaa.dev/support".into(),
            server_ban_permanent: "#f87171&lДОСТУП К СЕРВЕРУ ЗАБЛОКИРОВАН #94a3b8(Навсегда)\n\n#94a3b8Причина: #f8fafc{reason}\n#94a3b8Модератор: #f8fafc{actor}\n#94a3b8Правило: #f8fafc{rule}\n#94a3b8Дело: #64748b#{id}\n\n#64748bВы можете играть на других серверах сети.".into(),
            server_ban_temporary: "#f87171&lДОСТУП К СЕРВЕРУ ВРЕМЕННО ОГРАНИЧЕН\n\n#94a3b8Причина: #f8fafc{reason}\n#94a3b8Модератор: #f8fafc{actor}\n#94a3b8Окончание: #fbbf24{expires} #94a3b8(через #fbbf24{duration}#94a3b8)\n#94a3b8Правило: #f8fafc{rule}\n#94a3b8Дело: #64748b#{id}\n\n#64748bНа остальных серверах сети доступ сохранен.".into(),
            mute_permanent: "#f87171&l[!] #f87171Ваш чат заблокирован навсегда. #94a3b8Причина: #f8fafc{reason} #94a3b8(Выдал: #f8fafc{actor}#94a3b8)".into(),
            mute_temporary: "#f87171&l[!] #f87171Ваш чат заблокирован еще на #fbbf24{duration}#f87171. #94a3b8Причина: #f8fafc{reason} #94a3b8(Выдал: #f8fafc{actor}#94a3b8)".into(),
            warn_notice: "#fbbf24&l[!] #fbbf24Вам выдано предупреждение от #f8fafc{actor}#fbbf24. #94a3b8Причина: #f8fafc{reason} #94a3b8[#f8fafc{rule}#94a3b8]".into(),
            broadcast: "#f87171&l[Модерация] #f8fafc{player} #94a3b8получил наказание (#f87171{kind}#94a3b8) от #f8fafc{actor}#94a3b8: #f8fafc{reason}".into(),
            actor_receipt: "#4ade80&l[Успешно] #f8fafc{player} #94a3b8наказан (#4ade80{kind}#94a3b8): #f8fafc{reason}".into(),
        }
    }
}

/// Прочитать шаблоны. Битое значение в базе не должно ронять вход игрока —
/// тогда берём умолчания и говорим об этом в лог.
pub async fn load(pool: &sqlx::PgPool) -> ModerationMessages {
    let stored = match crate::db::get_setting(pool, SETTINGS_KEY).await {
        Ok(Some(value)) => value,
        Ok(None) => return ModerationMessages::default(),
        Err(e) => {
            tracing::warn!(error = %e, "шаблоны наказаний не читаются, беру умолчания");
            return ModerationMessages::default();
        }
    };
    serde_json::from_value(stored).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "шаблоны наказаний испорчены, беру умолчания");
        ModerationMessages::default()
    })
}

/// `GET /api/agent/messages` — агент читает шаблоны при старте и по сигналу
/// `messages_changed`.
pub async fn agent_messages(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _agent: crate::api::auth::AgentAuth,
) -> axum::Json<ModerationMessages> {
    axum::Json(load(&state.db).await)
}
