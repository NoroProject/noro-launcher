//! Каталог модов: единый вход в Modrinth и CurseForge.
//!
//! Оба API отдают одно и то же разной формой, поэтому нормализуем здесь, а не в
//! админке. Веб получает один тип карточки и одну кнопку установки — какой из
//! источников за ней стоит, ему знать не нужно.

pub mod cache;
pub mod curseforge;
mod curseforge_ids;
mod curseforge_map;
pub mod json;
pub mod modrinth;
mod modrinth_map;
pub mod query;
pub mod resolve;
pub mod types;

#[cfg(test)]
mod curseforge_tests;
#[cfg(test)]
mod modrinth_tests;
#[cfg(test)]
mod resolve_tests;

pub use cache::HttpCache;
pub use query::{SearchPage, SearchQuery};
pub use types::{Category, ModHit, ModProject, ModSource, ModVersion, Provider, ResolvedMod};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde_json::Value;
use std::sync::Arc;

pub fn provider_of(name: &str) -> AppResult<Provider> {
    match name {
        "modrinth" => Ok(Provider::Modrinth),
        "curseforge" => Ok(Provider::Curseforge),
        other => Err(AppError::BadRequest(format!(
            "неизвестный провайдер каталога: {other}"
        ))),
    }
}

/// GET с кешем поверх. Ключ — сам URL: он полностью описывает ответ.
pub async fn fetch_json(
    state: &AppState,
    url: &str,
    api_key: Option<&str>,
) -> AppResult<Arc<Value>> {
    if let Some(cached) = state.catalog.get(url) {
        return Ok(cached);
    }
    let mut req = state.http().get(url);
    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
    }
    let resp = req.send().await.map_err(|e| AppError::Other(e.into()))?;
    let status = resp.status();
    let body = resp.text().await.map_err(|e| AppError::Other(e.into()))?;
    if !status.is_success() {
        // Тело каталога бывает страницей на несколько килобайт — в сообщение
        // об ошибке идёт только начало.
        let head: String = body.chars().take(200).collect();
        return Err(AppError::BadRequest(format!(
            "каталог ответил {status}: {head}"
        )));
    }
    let value: Value = serde_json::from_str(&body).map_err(|e| AppError::Other(e.into()))?;
    Ok(state.catalog.put(url.to_string(), value))
}

pub async fn search(state: &AppState, q: &SearchQuery) -> AppResult<SearchPage> {
    if q.provider != "all" {
        return match provider_of(&q.provider)? {
            Provider::Modrinth => modrinth::search(state, q).await,
            Provider::Curseforge => curseforge::search(state, q).await,
        };
    }

    let (mr, cf) = tokio::join!(modrinth::search(state, q), curseforge::search(state, q));
    let mut failed = Vec::new();
    let mr = unwrap_or_note(mr, "modrinth", &mut failed);
    let cf = unwrap_or_note(cf, "curseforge", &mut failed);
    if failed.len() == 2 {
        return Err(AppError::BadRequest("ни один каталог не ответил".into()));
    }

    Ok(SearchPage {
        total: mr.total + cf.total,
        hits: interleave(mr.hits, cf.hits),
        failed,
    })
}

pub async fn project(state: &AppState, provider: Provider, id: &str) -> AppResult<ModProject> {
    match provider {
        Provider::Modrinth => modrinth::project(state, id).await,
        Provider::Curseforge => curseforge::project(state, id).await,
    }
}

pub async fn versions(
    state: &AppState,
    provider: Provider,
    id: &str,
    mc: Option<&str>,
    loader: Option<&str>,
) -> AppResult<Vec<ModVersion>> {
    match provider {
        Provider::Modrinth => modrinth::versions(state, id, mc, loader).await,
        Provider::Curseforge => curseforge::versions(state, id, mc, loader).await,
    }
}

pub async fn categories(
    state: &AppState,
    provider: Provider,
    project_type: &str,
) -> AppResult<Vec<Category>> {
    match provider {
        Provider::Modrinth => modrinth::categories(state, project_type).await,
        Provider::Curseforge => curseforge::categories(state, project_type).await,
    }
}

fn unwrap_or_note(
    result: AppResult<SearchPage>,
    name: &str,
    failed: &mut Vec<String>,
) -> SearchPage {
    result.unwrap_or_else(|e| {
        tracing::warn!(provider = name, error = %e, "каталог не ответил");
        failed.push(name.to_string());
        SearchPage {
            hits: Vec::new(),
            total: 0,
            failed: Vec::new(),
        }
    })
}

/// Чередуем выдачи, а не сортируем: релевантность Modrinth и «популярность»
/// CurseForge — разные шкалы, и общий порядок из них не собрать. Зато у обоих
/// источников равные шансы попасть в первый экран.
fn interleave(a: Vec<ModHit>, b: Vec<ModHit>) -> Vec<ModHit> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut a, mut b) = (a.into_iter(), b.into_iter());
    loop {
        match (a.next(), b.next()) {
            (None, None) => return out,
            (first, second) => out.extend(first.into_iter().chain(second)),
        }
    }
}

/// Найти последнюю совместимую версию мода и вернуть `ModSource` для resolve.
pub async fn source_for_project(
    state: &AppState,
    provider_name: &str,
    project_id: &str,
    mc_version: &str,
    modloader: &str,
) -> AppResult<ModSource> {
    let provider = provider_of(provider_name)?;
    let vers = versions(state, provider, project_id, Some(mc_version), Some(modloader)).await?;
    let ver = vers
        .first()
        .ok_or_else(|| AppError::BadRequest("нет совместимых версий".into()))?;

    match provider {
        Provider::Modrinth => Ok(ModSource::Modrinth {
            version_id: ver.id.clone(),
        }),
        Provider::Curseforge => {
            let pid: u64 = project_id
                .parse()
                .map_err(|_| AppError::BadRequest("некорректный CurseForge project_id".into()))?;
            let fid: u64 = ver
                .id
                .parse()
                .map_err(|_| AppError::BadRequest("некорректный CurseForge file_id".into()))?;
            Ok(ModSource::Curseforge {
                project_id: pid,
                file_id: fid,
            })
        }
    }
}
