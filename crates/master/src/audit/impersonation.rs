//! Детальный аудит действий, совершённых под impersonation.
//!
//! Прямое следствие двух решений: под impersonation доступно всё, что доступно
//! игроку, и игроку об этом не сообщают. Значит в игровых логах и в истории
//! аккаунта действия админа неотличимы от действий самого игрока, и разрешить
//! спор («меня обокрали» / «это был не я») можно будет только изнутри.
//!
//! Поэтому здесь пишется не факт входа, а каждое изменяющее обращение: метод,
//! путь и статус. Вне impersonation такой детализации не нужно — там достаточно
//! событий уровня «изменил роль», и их пишут сами хендлеры.

use crate::state::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use serde_json::json;
use uuid::Uuid;

/// Читающие запросы не пишем: под impersonation их сотни на каждый экран, а
/// спор разрешают изменения.
fn is_mutating(method: &axum::http::Method) -> bool {
    !matches!(
        *method,
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    )
}

pub async fn layer(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if !is_mutating(req.method()) {
        return next.run(req).await;
    }

    // Всё нужное достаём до первого `await`: тело запроса не `Sync`, и ссылка
    // на него через точку ожидания делает будущее не-`Send`.
    let token = bearer_uuid(&req);
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let actor = match token {
        Some(t) => impersonator(&state, t).await,
        None => None,
    };
    let response = next.run(req).await;

    if let Some((actor_id, target_id)) = actor {
        let username = crate::db::get_user(&state.db, actor_id)
            .await
            .ok()
            .flatten()
            .map(|u| u.mc_username)
            .unwrap_or_else(|| actor_id.to_string());

        crate::audit::record(
            &state,
            &crate::audit::Actor::User {
                id: actor_id,
                username,
            },
            "impersonate.action",
            crate::audit::target("user", target_id),
            json!({
                "method": method,
                "path": path,
                "status": response.status().as_u16(),
            }),
        )
        .await;
    }

    response
}

/// Bearer-токен запроса, если он похож на пользовательскую сессию.
fn bearer_uuid(req: &Request) -> Option<Uuid> {
    let raw = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?
        .trim();
    Uuid::parse_str(raw).ok()
}

/// `(кто на самом деле, от чьего имени)`, если запрос идёт под impersonation.
async fn impersonator(state: &AppState, token: Uuid) -> Option<(Uuid, Uuid)> {
    let actor_id = crate::db::session_impersonated_by(&state.db, token)
        .await
        .ok()
        .flatten()?;
    let target = crate::db::user_by_access_token(&state.db, token)
        .await
        .ok()
        .flatten()?;
    Some((actor_id, target.id))
}

#[cfg(test)]
#[path = "impersonation_tests.rs"]
mod tests;
