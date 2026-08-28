//! Storage cleanup: objects nothing references any more.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::PERM_STORAGE;

/// Count orphans without touching them. The sweep reads the whole database and
/// the whole storage tree, so it's triggered by hand rather than on a schedule.
pub async fn scan_orphans(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<crate::files::gc::Report>> {
    admin.require(PERM_STORAGE)?;
    let report = crate::files::gc::collect(&state.db, &state.files, false).await?;
    Ok(Json(report))
}

/// The same sweep, but it deletes. Kept on its own method so a page refresh or
/// a repeated GET can't wipe the store.
pub async fn delete_orphans(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<crate::files::gc::Report>> {
    admin.require(PERM_STORAGE)?;
    let report = crate::files::gc::collect(&state.db, &state.files, true).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::STORAGE_GC,
        None,
        serde_json::json!({
            "orphan_count": report.orphan_count,
            "orphan_bytes": report.orphan_bytes,
        }),
    )
    .await;
    tracing::warn!(
        count = report.orphan_count,
        bytes = report.orphan_bytes,
        "deleted unreferenced storage objects"
    );
    Ok(Json(report))
}
