use crate::directories::LauncherDirectories;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Fetched from upstream, not from the master, and only once — the jar is
/// version-independent and every instance uses the same copy.
pub async fn ensure_authlib_injector(
    client: &reqwest::Client,
    dirs: &LauncherDirectories,
) -> Result<PathBuf> {
    let path = dirs.authlib_injector();
    if path.exists() {
        return Ok(path);
    }

    tracing::info!("downloading authlib-injector");
    let meta: serde_json::Value = client
        .get("https://authlib-injector.yushi.moe/artifact/latest.json")
        .send()
        .await?
        .json()
        .await?;
    let url = meta["download_url"]
        .as_str()
        .ok_or_else(|| anyhow!("authlib-injector metadata has no download_url"))?;
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
