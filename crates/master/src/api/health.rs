//! Health-check для оркестратора и мониторинга.

use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

/// `GET /health` — жив ли мастер и отвечает ли БД.
///
/// Проверка идёт до базы намеренно: процесс, поднятый без работающего
/// PostgreSQL, отвечает на HTTP, но не может обслужить ни один запрос.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") })),
        ),
        Err(e) => {
            tracing::error!(error = %e, "health-check: БД недоступна");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "status": "db_unavailable" })),
            )
        }
    }
}
