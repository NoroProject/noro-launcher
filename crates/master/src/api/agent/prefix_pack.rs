//! Badge pack for the agent: where to fetch it and which glyph belongs to which
//! role. The agent builds the chat prefix from this and serves the pack to
//! players; details in `docs/prefix-pack-plan.md`.

use crate::api::auth::AgentAuth;
use crate::error::AppResult;
use crate::prefix;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;

/// `GET /api/agent/prefix-pack`
///
/// Serves whatever was built last. Rebuilding is a separate admin action:
/// roles are edited in batches, and a new pack after every edit would make
/// players reload resources ten times in a row.
pub async fn current(
    State(state): State<AppState>,
    _auth: AgentAuth,
) -> AppResult<Json<prefix::Current>> {
    Ok(Json(prefix::current(&state).await?))
}
