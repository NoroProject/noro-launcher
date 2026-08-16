//! Админ: состояние лаунчера игрока и его сессии.

use crate::api::auth::AdminAuth;
use crate::audit::{self};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::PERM_ADMIN_USERS;
use serde_json::{json, Value};
use uuid::Uuid;

/// Что видно на карточке игрока: онлайн, версия, актуальность.
pub async fn launcher_status(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_USERS)?;

    let client = crate::db::launcher_client(&state.db, id).await?;
    // Версия сравнивается безотносительно платформы: клиент сообщает её как
    // `macos-aarch64`, а в launcher_versions лежит target-триплет. Релиз всё
    // равно выкатывается одной версией на все платформы.
    let current = crate::db::current_launcher_version_any(&state.db).await?;

    Ok(Json(json!({
        "online": state.ws.is_user_connected(id),
        "version": client.as_ref().map(|c| c.version.clone()),
        "platform": client.as_ref().map(|c| c.platform.clone()),
        "last_seen_at": client.as_ref().map(|c| c.last_seen_at),
        "current_version": current,
        "outdated": match (&client, &current) {
            (Some(c), Some(v)) => &c.version != v,
            _ => false,
        },
        "open_integrity_flags": crate::db::open_integrity_flag_count(&state.db, id).await?,
    })))
}

/// Активные сессии игрока.
pub async fn sessions(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_USERS)?;
    let rows = crate::db::list_sessions(&state.db, id, None).await?;
    Ok(Json(json!(rows)))
}

/// Завершить все сессии игрока.
///
/// Единственный способ отозвать скомпрометированный токен: раньше строка жила
/// до истечения срока, и «выйти на всех устройствах» означало ждать месяц.
pub async fn revoke_sessions(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_USERS)?;
    let revoked = crate::db::revoke_all_sessions(&state.db, id, None).await?;

    audit::record(
        &state,
        &admin.actor,
        "user.sessions.revoke",
        audit::target("user", id),
        json!({ "revoked": revoked }),
    )
    .await;

    Ok(Json(json!({ "revoked": revoked })))
}

/// Завершить одну сессию.
pub async fn revoke_session(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, session_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_USERS)?;
    if !crate::db::revoke_session(&state.db, session_id, Some(id)).await? {
        return Err(AppError::NotFound("сессия".into()));
    }
    audit::record(
        &state,
        &admin.actor,
        "user.session.revoke",
        audit::target("user", id),
        json!({ "session_id": session_id }),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}
