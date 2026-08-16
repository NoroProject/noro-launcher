//! Журнал админских действий.
//!
//! Пишется смысл события — «выдал роль», «выкатил версию» — вместе с тем, что
//! именно изменилось. Логировать каждый HTTP-запрос здесь незачем: из метода и
//! пути не восстановить, что стало с объектом, а разбирать спорную ситуацию
//! придётся именно по этому.

pub mod impersonation;

use crate::state::AppState;
use serde_json::Value;
use uuid::Uuid;

/// Кто совершил действие.
#[derive(Debug, Clone)]
pub enum Actor {
    User {
        id: Uuid,
        username: String,
    },
    /// Admin-токен из CLI/CI: пользователя за ним нет.
    Token {
        name: String,
    },
}

impl Actor {
    pub fn id(&self) -> Option<Uuid> {
        match self {
            Actor::User { id, .. } => Some(*id),
            Actor::Token { .. } => None,
        }
    }

    /// Подпись строкой: переживает удаление аккаунта, в отличие от `actor_id`.
    pub fn label(&self) -> String {
        match self {
            Actor::User { username, .. } => username.clone(),
            Actor::Token { name } => format!("admin-token:{name}"),
        }
    }
}

/// Что затронуто действием.
pub struct Target {
    pub kind: &'static str,
    pub id: String,
}

/// Объект действия: `target("user", id)`.
pub fn target(kind: &'static str, id: impl ToString) -> Option<Target> {
    Some(Target {
        kind,
        id: id.to_string(),
    })
}

/// Записать событие.
///
/// Ошибка записи не валит само действие: журнал важен, но отказ сохранить
/// строку — не повод откатывать уже сделанное. Поэтому только лог.
pub async fn record(
    state: &AppState,
    actor: &Actor,
    action: &str,
    target: Option<Target>,
    details: Value,
) {
    let (kind, id) = match &target {
        Some(t) => (Some(t.kind), Some(t.id.as_str())),
        None => (None, None),
    };
    if let Err(e) = crate::db::insert_audit(
        &state.db,
        actor.id(),
        &actor.label(),
        action,
        kind,
        id,
        &details,
        None,
    )
    .await
    {
        tracing::error!(error = %e, action, "не удалось записать событие в аудит");
    }
}
