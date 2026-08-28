use crate::directories::LauncherDirectories;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Скачать authlib-injector, если его ещё нет.
pub async fn ensure_authlib_injector(
    client: &reqwest::Client,
    dirs: &LauncherDirectories,
) -> Result<PathBuf> {
    let path = dirs.authlib_injector();
    if path.exists() {
        return Ok(path);
    }

    tracing::info!("скачивание authlib-injector");
    let meta: serde_json::Value = client
        .get("https://authlib-injector.yushi.moe/artifact/latest.json")
        .send()
        .await?
        .json()
        .await?;
    let url = meta["download_url"]
        .as_str()
        .ok_or_else(|| anyhow!("нет download_url для authlib-injector"))?;
    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, &bytes).await?;
    Ok(path)
}
