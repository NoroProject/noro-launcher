//! Админ: очередь дел и карточка разбора.
//!
//! Карточка собирается одним запросом: лента, жалобы, наказания и срез чата
//! нужны вместе, а четыре обращения подряд на открытие дела означали бы, что
//! модератор смотрит на пустые панели, пока они догружаются.

mod actions;
mod attachments;
mod dossier;
mod probes;
mod quote;

pub use actions::*;
pub use attachments::*;
pub use dossier::*;
pub use probes::*;
pub use quote::*;

use crate::api::auth::AdminAuth;
use crate::api::paging::{flexible_i64, Page, PageQuery};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::{PERM_CASES_CHAT, PERM_CASES_VIEW};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CasesQuery {
    #[serde(default)]
    pub open_only: bool,
    /// Ник нарушителя, сервер, кто взял дело или его номер.
    pub q: Option<String>,
    #[serde(default, deserialize_with = "flexible_i64::deserialize")]
    pub limit: Option<i64>,
    #[serde(default, deserialize_with = "flexible_i64::deserialize")]
    pub offset: Option<i64>,
}

/// GET /api/admin/cases
///
/// Ищет и режет на страницы мастер. Раньше отдавалась вся очередь, а поиск жил
/// у клиента — и админка, и панель в игре искали каждая по своей копии, то есть
/// только по тому, что успело загрузиться.
pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(query): Query<CasesQuery>,
) -> AppResult<Json<Page<crate::db::cases::CaseListRow>>> {
    admin.require(PERM_CASES_VIEW)?;
    let page = PageQuery::from_parts(query.q, query.limit, query.offset);
    let like = page.like();
    let (items, total) = crate::db::list_cases(
        &state.db,
        query.open_only,
        like.as_deref(),
        page.limit(),
        page.offset(),
    )
    .await?;
    Ok(Json(Page::new(items, total)))
}

/// GET /api/admin/cases/{id}
pub async fn get(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_VIEW)?;
    let case = crate::db::get_case_view(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("case".into()))?;

    let reports = crate::db::case_reports(&state.db, id).await?;
    let events = crate::db::list_events(&state.db, id).await?;
    let punishments = crate::db::punishments::list_by_case(&state.db, id).await?;

    // Срез чата — отдельное право: в очереди дел переписки нет, а тут есть.
    let chat_allowed = admin.require(PERM_CASES_CHAT).is_ok();
    let messages = if chat_allowed {
        crate::db::list_messages(&state.db, id).await?
    } else {
        Vec::new()
    };

    // Репутация каждого жалобщика: десять жалоб от того, у кого не подтвердилась
    // ни одна, читаются иначе, чем одна от того, у кого подтверждались все.
    let mut reporters = serde_json::Map::new();
    for report in &reports {
        if reporters.contains_key(&report.reporter_id.to_string()) {
            continue;
        }
        let stats = crate::db::reporter_stats(&state.db, report.reporter_id).await?;
        reporters.insert(report.reporter_id.to_string(), json!(stats));
    }

    Ok(Json(json!({
        "case": case,
        "reports": reports,
        "events": events,
        "punishments": punishments,
        "messages": messages,
        "chat_allowed": chat_allowed,
        "reporters": reporters,
    })))
}
