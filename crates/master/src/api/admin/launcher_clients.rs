//! Статистика по установленным лаунчерам.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::PERM_ADMIN_LAUNCHER;
use std::collections::BTreeMap;

#[derive(serde::Serialize)]
pub struct ClientsReport {
    /// Версия, которую мастер сейчас раздаёт. `None` — ни одна не выкачена.
    pub current_version: Option<String>,
    pub total: usize,
    /// Сколько сидит не на текущей версии — те, к кому вопросы «а почему не работает».
    pub outdated: usize,
    /// Сколько клиентов на каждой версии и платформе.
    pub by_version: BTreeMap<String, usize>,
    pub by_platform: BTreeMap<String, usize>,
    pub clients: Vec<crate::db::LauncherClientRow>,
}

/// `GET /api/admin/launcher/clients`
pub async fn clients(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<ClientsReport>> {
    admin.require(PERM_ADMIN_LAUNCHER)?;

    let clients = crate::db::list_launcher_clients(&state.db).await?;
    let current_version = crate::db::current_launcher_version_any(&state.db).await?;

    let mut by_version: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_platform: BTreeMap<String, usize> = BTreeMap::new();
    let mut outdated = 0;

    for c in &clients {
        // Пустая версия — лаунчер, выпущенный до того, как её начали слать.
        let version = if c.version.is_empty() {
            "unknown".to_string()
        } else {
            c.version.clone()
        };
        if let Some(current) = &current_version {
            if &version != current {
                outdated += 1;
            }
        }
        *by_version.entry(version).or_default() += 1;

        let platform = if c.platform.is_empty() {
            "unknown".to_string()
        } else {
            c.platform.clone()
        };
        *by_platform.entry(platform).or_default() += 1;
    }

    Ok(Json(ClientsReport {
        current_version,
        total: clients.len(),
        outdated,
        by_version,
        by_platform,
        clients,
    }))
}
