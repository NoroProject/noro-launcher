//! Уборка хранилища: объекты, на которые больше никто не ссылается.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::PERM_ADMIN_STORAGE;

/// `GET /api/admin/storage/orphans` — только посчитать, ничего не трогая.
///
/// Обход читает всю БД и весь каталог хранилища, поэтому запускается вручную,
/// а не по расписанию.
pub async fn scan_orphans(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<crate::files::gc::Report>> {
    admin.require(PERM_ADMIN_STORAGE)?;
    let report = crate::files::gc::collect(&state.db, &state.files, false).await?;
    Ok(Json(report))
}

/// `DELETE /api/admin/storage/orphans` — то же самое, но с удалением.
///
/// Отдельный метод намеренно: удаление не должно случаться от обновления
/// страницы или повторного GET.
pub async fn delete_orphans(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<crate::files::gc::Report>> {
    admin.require(PERM_ADMIN_STORAGE)?;
    let report = crate::files::gc::collect(&state.db, &state.files, true).await?;
    audit::record(
        &state,
        &admin.actor,
        "storage.gc",
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
        "удалены неиспользуемые объекты хранилища"
    );
    Ok(Json(report))
}
