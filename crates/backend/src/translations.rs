//! Translation catalogs from the master, cached on disk.
//!
//! The cache isn't about bandwidth — catalogs are a few KB. It's so the right
//! language is up before the network is: without it the UI shows English until
//! the master answers.

use crate::backend::Ctx;
use bridge::MessageToFrontend;
use std::path::PathBuf;

/// Sends the cached catalog straight away, then a fresh one from the master if
/// it differs. The frontend can receive `LocaleCatalog` twice for one call.
pub fn refresh(ctx: &Ctx, code: String) {
    let cached = read_cache(&cache_path(ctx, &code));
    if let Some(ftl) = &cached {
        ctx.send(MessageToFrontend::LocaleCatalog {
            code: code.clone(),
            ftl: ftl.clone(),
        });
    }

    let ctx = ctx.clone();
    tokio::spawn(async move {
        match fetch(&ctx, &code).await {
            Ok(Some(ftl)) => {
                if cached.as_deref() == Some(ftl.as_str()) {
                    return;
                }
                write_cache(&cache_path(&ctx, &code), &ftl);
                ctx.send(MessageToFrontend::LocaleCatalog { code, ftl });
            }
            Ok(None) => {
                tracing::debug!(locale = %code, "master has no catalog, keeping the built-in one");
            }
            Err(e) => {
                tracing::warn!(locale = %code, error = %e, "could not fetch translation catalog");
            }
        }
    });
}

async fn fetch(ctx: &Ctx, code: &str) -> anyhow::Result<Option<String>> {
    let url = format!(
        "{}/api/launcher/locales/{code}",
        ctx.config.get().master_url.trim_end_matches('/')
    );
    let resp = ctx.http.get(&url).send().await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let body: serde_json::Value = resp.error_for_status()?.json().await?;
    Ok(body["ftl"].as_str().map(str::to_string))
}

fn cache_path(ctx: &Ctx, code: &str) -> PathBuf {
    ctx.dirs.root().join("locales").join(format!("{code}.ftl"))
}

fn read_cache(path: &PathBuf) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn write_cache(path: &PathBuf, ftl: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(path, ftl) {
        tracing::warn!(error = %e, "could not write catalog cache");
    }
}
