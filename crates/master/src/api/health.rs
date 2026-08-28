//! Health check for the orchestrator and monitoring.

use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

/// `GET /health`
///
/// Reaches the database on purpose: a process started without a working
/// PostgreSQL still answers HTTP but can't serve a single request.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") })),
        ),
        Err(e) => {
            tracing::error!(error = %e, "health check: database unavailable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "status": "db_unavailable" })),
            )
        }
    }
}
