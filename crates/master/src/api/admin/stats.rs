use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

pub async fn stats(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Json<Value>> {
    admin.require(schema::PERM_ADMIN_STATS)?;
    let s = crate::db::stats(&state.db).await?;
    Ok(Json(json!({
        "users": s.users,
        "servers": s.servers,
        "builds": s.builds,
        "total_playtime_secs": s.total_playtime_secs,
        "online_launchers": state.ws.connected_count(),
        "authed_launchers": state.ws.authed_users(),
        "file_store_bytes": state.files.total_size().await,
    })))
}
