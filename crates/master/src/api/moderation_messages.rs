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
            ban_permanent: "You are banned from this network.\n\nReason: {reason}\nBy: {actor}\nCase: {id}".into(),
            ban_temporary: "You are banned from this network.\n\nReason: {reason}\nExpires: {expires} (in {duration})\nBy: {actor}\nCase: {id}".into(),
            server_ban_permanent: "You are banned from this server.\n\nReason: {reason}\nBy: {actor}\nCase: {id}".into(),
            server_ban_temporary: "You are banned from this server.\n\nReason: {reason}\nExpires: {expires} (in {duration})\nBy: {actor}\nCase: {id}".into(),
            mute_permanent: "You are muted. Reason: {reason}".into(),
            mute_temporary: "You are muted for another {duration}. Reason: {reason}".into(),
            warn_notice: "You have been warned by {actor}. Reason: {reason}".into(),
            broadcast: "{player} was {kind} by {actor}: {reason}".into(),
            actor_receipt: "{player} was {kind}: {reason}".into(),
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
