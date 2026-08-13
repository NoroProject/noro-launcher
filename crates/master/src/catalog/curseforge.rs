//! Запросы к CurseForge. Без `CURSEFORGE_API_KEY` провайдер просто выключен.

use super::curseforge_ids::{class_id, loader_type, sort_field};
use super::json::{items, num};
use super::query::{SearchPage, SearchQuery};
use super::types::{Category, ModProject, ModVersion};
use super::{curseforge_map as map, fetch_json};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use urlencoding::encode;

const API: &str = "https://api.curseforge.com/v1";
/// Minecraft в номенклатуре CurseForge.
const GAME_ID: u32 = 432;

fn api_key(state: &AppState) -> AppResult<&str> {
    state
        .config
        .curseforge_api_key
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("CURSEFORGE_API_KEY не задан".into()))
}

pub async fn search(state: &AppState, q: &SearchQuery) -> AppResult<SearchPage> {
    let key = api_key(state)?;
    let mut url = format!(
        "{API}/mods/search?gameId={GAME_ID}&classId={}&searchFilter={}&sortField={}&sortOrder=desc&index={}&pageSize={}",
        class_id(&q.project_type),
        encode(&q.q),
        sort_field(&q.sort),
        q.offset,
        q.page_size()
    );
    if let Some(mc) = q.mc.as_deref().filter(|v| !v.is_empty()) {
        url.push_str(&format!("&gameVersion={}", encode(mc)));
    }
    if let Some(loader) = q.loader.as_deref().and_then(loader_type) {
        url.push_str(&format!("&modLoaderType={loader}"));
    }
    // Категории у CurseForge — числовые id; текстовый slug тут не принимается,
    // поэтому чужие значения (например, категории Modrinth) просто игнорируем.
    if let Some(id) = q.category_list().iter().find_map(|c| c.parse::<u32>().ok()) {
        url.push_str(&format!("&categoryId={id}"));
    }

    let resp = fetch_json(state, &url, Some(key)).await?;
    Ok(SearchPage {
        hits: items(&resp, "data").iter().map(map::hit).collect(),
        total: num(&resp["pagination"], "totalCount"),
        failed: Vec::new(),
    })
}

pub async fn project(state: &AppState, id: &str) -> AppResult<ModProject> {
    let key = api_key(state)?;
    let id = numeric(id)?;
    let meta = fetch_json(state, &format!("{API}/mods/{id}"), Some(key)).await?;
    // Описание отдельным запросом: в карточке мода его нет, только summary.
    let body = fetch_json(state, &format!("{API}/mods/{id}/description"), Some(key))
        .await
        .ok()
        .and_then(|v| v["data"].as_str().map(String::from))
        .unwrap_or_default();
    Ok(map::project(&meta["data"], body))
}

pub async fn versions(
    state: &AppState,
    id: &str,
    mc: Option<&str>,
    loader: Option<&str>,
) -> AppResult<Vec<ModVersion>> {
    let key = api_key(state)?;
    let id = numeric(id)?;
    let mut url = format!("{API}/mods/{id}/files?pageSize=50");
    if let Some(mc) = mc.filter(|v| !v.is_empty()) {
        url.push_str(&format!("&gameVersion={}", encode(mc)));
    }
    if let Some(t) = loader.and_then(loader_type) {
        url.push_str(&format!("&modLoaderType={t}"));
    }
    let resp = fetch_json(state, &url, Some(key)).await?;
    Ok(items(&resp, "data").iter().map(map::version).collect())
}

pub async fn categories(state: &AppState, project_type: &str) -> AppResult<Vec<Category>> {
    let key = api_key(state)?;
    let url = format!(
        "{API}/categories?gameId={GAME_ID}&classId={}",
        class_id(project_type)
    );
    let resp = fetch_json(state, &url, Some(key)).await?;
    let mut list: Vec<Category> = items(&resp, "data").iter().map(map::category).collect();
    list.sort_by(|a, b| a.display.cmp(&b.display));
    Ok(list)
}

/// Ссылка на скачивание файла. Отдельный вызов: в карточке файла её может не
/// быть, если автор запретил стороннюю загрузку.
pub async fn download_url(state: &AppState, project_id: u64, file_id: u64) -> AppResult<String> {
    let key = api_key(state)?;
    let url = format!("{API}/mods/{project_id}/files/{file_id}/download-url");
    let resp = fetch_json(state, &url, Some(key)).await?;
    resp["data"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AppError::BadRequest("мод запретил стороннее скачивание".into()))
}

pub async fn file_meta(
    state: &AppState,
    project_id: u64,
    file_id: u64,
) -> AppResult<serde_json::Value> {
    let key = api_key(state)?;
    let url = format!("{API}/mods/{project_id}/files/{file_id}");
    Ok(fetch_json(state, &url, Some(key)).await?["data"].clone())
}

fn numeric(id: &str) -> AppResult<u64> {
    id.parse()
        .map_err(|_| AppError::BadRequest(format!("id CurseForge должен быть числом, а не {id}")))
}
