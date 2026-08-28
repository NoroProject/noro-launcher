//! Stats over installed launchers.

use crate::api::auth::AdminAuth;
use crate::api::paging::PageQuery;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use schema::PERM_LAUNCHER_CLIENTS;

use std::collections::BTreeMap;

#[derive(serde::Serialize)]
pub struct ClientsReport {
    /// What the master is serving right now. `None` if nothing is rolled out.
    pub current_version: Option<String>,
    pub total: i64,
    /// Clients on anything other than `current_version`.
    pub outdated: i64,
    pub by_version: BTreeMap<String, i64>,
    pub by_platform: BTreeMap<String, i64>,
    pub clients: Vec<crate::db::LauncherClientRow>,
}

/// The totals and the breakdowns are counted in the database, over every row;
/// only `clients` is a page. Deriving them from the page instead gives numbers
/// that look plausible and are wrong once there are more installs than fit.
pub async fn clients(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<ClientsReport>> {
    admin.require(PERM_LAUNCHER_CLIENTS)?;

    let (clients, total) =
        crate::db::list_launcher_clients(&state.db, page.limit(), page.offset()).await?;
    let current_version = crate::db::current_launcher_version_any(&state.db).await?;

    let outdated = match &current_version {
        Some(current) => crate::db::launcher_clients_outdated(&state.db, current).await?,
        // Nothing rolled out, so nothing to be behind.
        None => 0,
    };

    Ok(Json(ClientsReport {
        current_version,
        total,
        outdated,
        by_version: counts(&state, "version").await?,
        by_platform: counts(&state, "platform").await?,
        clients,
    }))
}

/// Breakdown by column. Clients from before a column was reported come back as
/// `unknown` rather than an empty string.
async fn counts(state: &AppState, column: &str) -> AppResult<BTreeMap<String, i64>> {
    Ok(crate::db::launcher_client_counts(&state.db, column)
        .await?
        .into_iter()
        .collect())
}
