//! Статистика по установленным лаунчерам.

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
    /// Версия, которую мастер сейчас раздаёт. `None` — ни одна не выкачена.
    pub current_version: Option<String>,
    pub total: i64,
    /// Сколько сидит не на текущей версии — те, к кому вопросы «а почему не работает».
    pub outdated: i64,
    /// Сколько клиентов на каждой версии и платформе.
    pub by_version: BTreeMap<String, i64>,
    pub by_platform: BTreeMap<String, i64>,
    pub clients: Vec<crate::db::LauncherClientRow>,
}

/// `GET /api/admin/launcher/clients`
///
/// Сводка считается по всей таблице, а список отдаётся страницей. Раньше и то и
/// другое бралось из одной выдачи на 500 строк: при большем числе установок
/// «всего» упиралось в 500, а разбивка по версиям строилась по случайным пятистам
/// — цифры выглядели правдоподобно и врали.
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
        // Ни одна версия не выкачена — «отстающих» не с чем сравнивать.
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

/// Разбивка по колонке. Пустое значение — клиент, выпущенный до того, как её
/// начали слать; в отчёте это `unknown`, а не пустая строка.
async fn counts(state: &AppState, column: &str) -> AppResult<BTreeMap<String, i64>> {
    Ok(crate::db::launcher_client_counts(&state.db, column)
        .await?
        .into_iter()
        .collect())
}
