//! Admin action log.
//!
//! What gets written is the meaning of the event — "granted a role", "deployed
//! a version" — together with what actually changed. Logging every HTTP request
//! would not help: a method and a path don't say what became of the object, and
//! that's what a dispute is settled on.

pub mod actions;
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

/// Событие, совершённое самим игроком: вход, запуск игры, сверка.
///
/// Отдельный хелпер, потому что «кто» тут известен по id, а имя приходится
/// доставать — и забыть его значит получить в журнале строку без автора.
pub async fn record_by_user(
    state: &AppState,
    user_id: Uuid,
    action: &'static actions::Action,
    details: Value,
) {
    let username = crate::db::get_user(&state.db, user_id)
        .await
        .ok()
        .flatten()
        .map(|u| u.mc_username)
        .unwrap_or_else(|| user_id.to_string());

    record(
        state,
        &Actor::User {
            id: user_id,
            username,
        },
        action,
        target("user", user_id),
        details,
    )
    .await;
}

/// Записать событие.
///
/// Ошибка записи не валит само действие: журнал важен, но отказ сохранить
/// строку — не повод откатывать уже сделанное. Поэтому только лог.
pub async fn record(
    state: &AppState,
    actor: &Actor,
    action: &'static actions::Action,
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
        action.name,
        kind,
        id,
        &details,
        None,
    )
    .await
    {
        tracing::error!(error = %e, action = action.name, "не удалось записать событие в аудит");
    }
}
