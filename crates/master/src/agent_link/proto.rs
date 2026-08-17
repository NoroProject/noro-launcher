//! Кадры канала мастер → агент игрового сервера.
//!
//! Канал односторонний по смыслу: наказания выдаются через обычный HTTP, а сюда
//! приходит только «вот что изменилось, примени сейчас». Обратно агент шлёт
//! разве что `pong` — держать соединение живым.
//!
//! Игрока адресуем MC UUID, а не `users.id`: агент знает игрока только таким, и
//! перекладывать сопоставление на него значит завести на игровой стороне вторую
//! копию базы.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Наказание в том виде, в каком его показывают игроку.
#[derive(Serialize, Clone, Debug)]
pub struct LivePunishment {
    pub id: Uuid,
    /// MC UUID наказанного.
    pub target: Uuid,
    pub target_name: String,
    /// `ban` | `server_ban` | `mute` | `warn`.
    pub kind: String,
    pub reason: String,
    pub actor_label: String,
    pub created_at: DateTime<Utc>,
    /// `None` — навсегда.
    pub expires_at: Option<DateTime<Utc>>,
    pub rule_code: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToAgent {
    /// Наказание выдано или продлено — применить немедленно.
    Punished { punishment: LivePunishment },
    /// Наказание снято. `kind` нужен, чтобы агент понял, что именно отпустить:
    /// снятый мут возвращает чат, снятый бан не делает ничего с онлайном.
    Revoked {
        id: Uuid,
        target: Uuid,
        target_name: String,
        kind: String,
        actor_label: String,
    },
    /// Шаблоны сообщений поменяли в админке — перечитать их с мастера.
    MessagesChanged,
}
