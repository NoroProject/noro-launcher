//! Админ-API каталога модов: поиск, карточка проекта, версии, фасеты.
//!
//! Ходить в Modrinth и CurseForge прямо из браузера нельзя: у CurseForge ключ
//! API, который в вебе светить незачем, а у обоих — CORS и рейт-лимит. Мастер
//! проксирует и кеширует.

use crate::api::auth::AdminAuth;
use crate::catalog::{self, Category, ModProject, ModVersion, SearchPage, SearchQuery};
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::PERM_ADMIN_BUILDS;
use serde::{Deserialize, Serialize};

pub async fn search(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<SearchQuery>,
) -> AppResult<Json<SearchPage>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    Ok(Json(catalog::search(&state, &q).await?))
}

pub async fn project(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((provider, id)): Path<(String, String)>,
) -> AppResult<Json<ModProject>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    let provider = catalog::provider_of(&provider)?;
    Ok(Json(catalog::project(&state, provider, &id).await?))
}

#[derive(Deserialize)]
pub struct VersionQuery {
    /// Пусто — значит «все версии игры»: в глобальном браузере сборки ещё нет.
    pub mc: Option<String>,
    pub loader: Option<String>,
}

pub async fn versions(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((provider, id)): Path<(String, String)>,
    Query(q): Query<VersionQuery>,
) -> AppResult<Json<Vec<ModVersion>>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    let provider = catalog::provider_of(&provider)?;
    let versions =
        catalog::versions(&state, provider, &id, q.mc.as_deref(), q.loader.as_deref()).await?;
    Ok(Json(versions))
}

#[derive(Deserialize)]
pub struct CategoryQuery {
    #[serde(default = "modrinth")]
    pub provider: String,
    #[serde(default = "mods")]
    pub project_type: String,
}

fn modrinth() -> String {
    "modrinth".into()
}
fn mods() -> String {
    "mod".into()
}

pub async fn categories(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<CategoryQuery>,
) -> AppResult<Json<Vec<Category>>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    let provider = catalog::provider_of(&q.provider)?;
    Ok(Json(
        catalog::categories(&state, provider, &q.project_type).await?,
    ))
}

#[derive(Serialize)]
pub struct Providers {
    /// Modrinth доступен всегда — ключа он не требует.
    pub modrinth: bool,
    pub curseforge: bool,
}

/// Какие источники реально работают. Без ключа CurseForge вкладку надо гасить,
/// а не показывать её и ронять поиск в ошибку на каждый запрос.
pub async fn providers(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Providers>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    Ok(Json(Providers {
        modrinth: true,
        curseforge: state.config.curseforge_api_key.is_some(),
    }))
}
