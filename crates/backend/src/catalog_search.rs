//! Mod catalog search against the master, launcher side.
//!
//! Every failure here has to reach the frontend as `CatalogFailed`. The catalog
//! screen has no timeout of its own — a request that returns nothing leaves it
//! spinning forever.

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
        .context("catalog is not answering")?
        .error_for_status()
        .context("catalog returned an error")?;
    let data: Value = res.json().await.context("catalog response is not json")?;

    let hits = data
        .get("hits")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("catalog response has no hits"))?
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

/// Drops hits with no provider or id rather than guessing a default: the
/// provider decides which API the install goes to.
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
