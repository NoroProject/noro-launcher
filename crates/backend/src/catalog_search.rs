//! Поиск модов в каталоге мастера — со стороны лаунчера.
//!
//! Любой обрыв здесь возвращается сообщением `CatalogFailed`. Раньше запрос
//! просто ничего не отправлял фронтенду, и экран каталога навсегда застывал на
//! «Searching compatible mods...» — ошибку не видел ни игрок, ни лог.

use anyhow::{anyhow, Context, Result};
use bridge::CatalogHitInfo;
use serde_json::Value;

pub struct SearchPage {
    pub hits: Vec<CatalogHitInfo>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

pub async fn search(
    http: &reqwest::Client,
    master_url: &str,
    query: &str,
    provider: &str,
    mc_version: Option<&str>,
    loader: Option<&str>,
    offset: u32,
) -> Result<SearchPage> {
    let mut url = format!(
        "{}/api/admin/catalog/search?q={}&provider={}&offset={offset}&limit=20",
        master_url.trim_end_matches('/'),
        urlencoding::encode(query),
        urlencoding::encode(provider),
    );
    if let Some(mc) = mc_version {
        url.push_str("&mc=");
        url.push_str(&urlencoding::encode(mc));
    }
    if let Some(ldr) = loader {
        url.push_str("&loader=");
        url.push_str(&urlencoding::encode(ldr));
    }

    let res = http
        .get(&url)
        .send()
        .await
        .context("каталог не отвечает")?
        .error_for_status()
        .context("каталог вернул ошибку")?;
    let data: Value = res.json().await.context("ответ каталога не разобрать")?;

    let hits = data
        .get("hits")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("в ответе каталога нет списка модов"))?
        .iter()
        .filter_map(hit)
        .collect();

    Ok(SearchPage {
        hits,
        total: u32_at(&data, "total").unwrap_or(0),
        offset: u32_at(&data, "offset").unwrap_or(offset),
        limit: u32_at(&data, "limit").unwrap_or(20),
    })
}

/// Мод без провайдера или id пропускаем.
///
/// Провайдер раньше по умолчанию считался modrinth: мод с CurseForge получал
/// чужую метку, и установка уходила в другой API — с ошибкой «не найдено».
fn hit(h: &Value) -> Option<CatalogHitInfo> {
    Some(CatalogHitInfo {
        provider: str_at(h, "provider")?,
        project_id: str_at(h, "project_id")?,
        title: str_at(h, "title")?,
        description: str_at(h, "description").unwrap_or_default(),
        icon_url: str_at(h, "icon_url"),
        author: str_at(h, "author"),
        downloads: h.get("downloads").and_then(Value::as_u64).unwrap_or(0),
    })
}

fn str_at(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

fn u32_at(v: &Value, key: &str) -> Option<u32> {
    v.get(key).and_then(Value::as_u64).map(|n| n as u32)
}
