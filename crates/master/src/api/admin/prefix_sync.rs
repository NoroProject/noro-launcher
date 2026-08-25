//! Пересобрать плашки по кнопке, а не на каждую правку роли.
//!
//! Раньше пак пересобирался при каждом обращении агента, то есть фактически на
//! каждый вход игрока. Админ правит роли пачкой — цвет, потом текст, потом
//! соседнюю роль, — и после каждой правки игроки получали новый пак и новую
//! перезагрузку ресурсов. Теперь момент выбирает человек.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::prefix;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::PERM_ROLES_EDIT;

/// `POST /api/admin/roles/sync-badges`
pub async fn sync(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<prefix::Current>> {
    admin.require(PERM_ROLES_EDIT)?;
    Ok(Json(prefix::rebuild(&state).await?))
}
