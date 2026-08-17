//! Заслонка `503 setup_required`.
//!
//! Ненастроенный мастер не должен делать вид, что работает: манифесты уехали бы
//! со ссылками в никуда, а OAuth-редирект вёл бы на пустую строку. Отвечать
//! честной ошибкой лучше, чем отдавать сломанные данные.

use crate::state::AppState;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Что доступно до завершения настройки.
///
/// `/health` — чтобы оркестратор не считал контейнер мёртвым и не перезапускал
/// его по кругу, `/api/setup/*` — собственно визард.
fn always_open(path: &str) -> bool {
    path == "/health" || path.starts_with("/api/setup/")
}

pub async fn gate(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if always_open(req.uri().path()) {
        return next.run(req).await;
    }

    match crate::db::instance_state(&state.db).await {
        Ok(st) if st.setup_completed => next.run(req).await,
        Ok(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "setup_required",
                "message": "this instance is not configured yet — open /setup",
            })),
        )
            .into_response(),
        // БД не отвечает: пропустить запрос дальше значит показать игроку
        // невнятную ошибку вместо внятной. Но и запирать инстанс из-за мигнувшей
        // БД нельзя — дальше по стеку тот же запрос упадёт сам и понятнее.
        Err(e) => {
            tracing::error!(error = %e, "не прочитать состояние установки");
            next.run(req).await
        }
    }
}

#[cfg(test)]
#[path = "gate_tests.rs"]
mod tests;
