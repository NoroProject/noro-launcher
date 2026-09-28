//! Mod catalog search against the master, launcher side.
//!
//! Goes to the player-facing `/api/catalog`, not the admin one: browsing
//! content is something every player does, and an endpoint under `/api/admin`
//! only worked because nobody had got round to checking permissions on it.
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

/// What the browser is asking for. A struct rather than eight arguments: half
/// of them are strings, and at eight positional strings a mistake stops being
/// catchable by the compiler.
pub struct Query<'a> {
    pub text: &'a str,
    pub provider: &'a str,
    pub mc_version: Option<&'a str>,
    pub loader: Option<&'a str>,
    /// `mod` · `resourcepack` · `shader`.
    pub project_type: &'a str,
    /// `relevance` · `downloads` · `follows` · `newest` · `updated`.
    pub sort: &'a str,
    pub offset: u32,
}

pub async fn search(
    http: &reqwest::Client,
    master_url: &str,
    token: Option<&str>,
    q: &Query<'_>,
) -> Result<SearchPage> {
    let enc = urlencoding::encode;
    let mut url = format!(
        "{}/api/catalog/search?q={}&provider={}&project_type={}&sort={}&offset={}&limit=20",
        master_url.trim_end_matches('/'),
        enc(q.text),
        enc(q.provider),
        enc(q.project_type),
        enc(q.sort),
        q.offset,
    );
    // Resource packs and shaders are published for a game version but not for a
    // loader, and sending one filters every result away.
    if let Some(mc) = q.mc_version {
        url.push_str("&mc=");
        url.push_str(&enc(mc));
    }
    if let (Some(loader), "mod") = (q.loader, q.project_type) {
        url.push_str("&loader=");
        url.push_str(&enc(loader));
    }

    let mut req = http.get(&url);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let res = req
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
        offset: u32_at(&data, "offset").unwrap_or(q.offset),
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
    v.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

fn u32_at(v: &Value, key: &str) -> Option<u32> {
    v.get(key).and_then(Value::as_u64).map(|n| n as u32)
}
