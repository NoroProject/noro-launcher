//! Пак плашек агенту: где его взять и какой символ у какой роли.
//!
//! Агент спрашивает это при старте и после каждой правки ролей. Из ответа он
//! собирает префикс компонентом и выдаёт пак игрокам — подробности в
//! `docs/prefix-pack-plan.md`.

use crate::api::auth::AgentAuth;
use crate::error::AppResult;
use crate::prefix;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;

/// `GET /api/agent/prefix-pack`
///
/// Отдаётся собранное в прошлый раз. Пересобирает пак админ кнопкой в списке
/// ролей: он правит роли пачкой, и рассылать новый пак после каждой правки —
/// значит заставлять игроков перезагружать ресурсы по десять раз подряд.
pub async fn current(
    State(state): State<AppState>,
    _auth: AgentAuth,
) -> AppResult<Json<prefix::Current>> {
    Ok(Json(prefix::current(&state).await?))
}
