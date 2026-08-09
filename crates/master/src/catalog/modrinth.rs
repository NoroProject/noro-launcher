//! Запросы к Modrinth.

use super::json::{items, num};
use super::query::{SearchPage, SearchQuery};
use super::types::{Category, ModProject, ModVersion};
use super::{fetch_json, modrinth_map as map};
use crate::error::AppResult;
use crate::state::AppState;
use urlencoding::encode;

const API: &str = "https://api.modrinth.com/v2";

pub async fn search(state: &AppState, q: &SearchQuery) -> AppResult<SearchPage> {
    let url = format!(
        "{API}/search?query={}&facets={}&index={}&offset={}&limit={}",
        encode(&q.q),
        encode(&facets(q)),
        index(&q.sort),
        q.offset,
        q.page_size()
    );
    let resp = fetch_json(state, &url, None).await?;
    Ok(SearchPage {
        hits: items(&resp, "hits").iter().map(map::hit).collect(),
        total: num(&resp, "total_hits"),
        failed: Vec::new(),
    })
}

pub async fn project(state: &AppState, id: &str) -> AppResult<ModProject> {
    let resp = fetch_json(state, &format!("{API}/project/{}", encode(id)), None).await?;
    Ok(map::project(&resp))
}

pub async fn versions(
    state: &AppState,
    id: &str,
    mc: Option<&str>,
    loader: Option<&str>,
) -> AppResult<Vec<ModVersion>> {
    let mut url = format!("{API}/project/{}/version", encode(id));
    // Фильтры Modrinth ждёт JSON-массивом прямо в query-строке.
    if let Some(mc) = mc.filter(|v| !v.is_empty()) {
        url.push_str(&format!("?game_versions={}", encode(&json_list(mc))));
    }
    if let Some(loader) = loader.filter(|v| !v.is_empty()) {
        let sep = if url.contains('?') { '&' } else { '?' };
        url.push_str(&format!("{sep}loaders={}", encode(&json_list(loader))));
    }
    let resp = fetch_json(state, &url, None).await?;
    Ok(resp
        .as_array()
        .map(|a| a.iter().filter_map(map::version).collect())
        .unwrap_or_default())
}

/// Категории для сайдбара фасетов — только те, что относятся к типу проекта.
pub async fn categories(state: &AppState, project_type: &str) -> AppResult<Vec<Category>> {
    let resp = fetch_json(state, &format!("{API}/tag/category"), None).await?;
    Ok(resp
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|c| c["project_type"].as_str() == Some(project_type))
                // header `resolutions`/`performance impact` — это про ресурспаки,
                // в фасетах модов им делать нечего.
                .filter(|c| c["header"].as_str() != Some("resolutions"))
                .map(map::category)
                .collect()
        })
        .unwrap_or_default())
}

fn json_list(value: &str) -> String {
    format!("[\"{value}\"]")
}

/// Фасеты — массив групп: внутри группы OR, между группами AND.
pub(super) fn facets(q: &SearchQuery) -> String {
    let mut groups = vec![format!("[\"project_type:{}\"]", q.project_type)];
    if let Some(mc) = q.mc.as_deref().filter(|v| !v.is_empty()) {
        groups.push(format!("[\"versions:{mc}\"]"));
    }
    if let Some(loader) = q.loader.as_deref().filter(|v| !v.is_empty()) {
        groups.push(format!("[\"categories:{loader}\"]"));
    }
    // Категории именно AND: выбрав «magic» и «technology», админ ищет мод,
    // который и то и другое, а не свалку из двух списков.
    for cat in q.category_list() {
        groups.push(format!("[\"categories:{cat}\"]"));
    }
    if let Some(side) = q.side.as_deref() {
        let field = if side == "server" {
            "server_side"
        } else {
            "client_side"
        };
        groups.push(format!("[\"{field}:required\",\"{field}:optional\"]"));
    }
    format!("[{}]", groups.join(","))
}

fn index(sort: &str) -> &str {
    match sort {
        "downloads" | "follows" | "newest" | "updated" => sort,
        _ => "relevance",
    }
}
