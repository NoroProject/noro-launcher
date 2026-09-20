use crate::config::LauncherConfig;
use crate::directories::LauncherDirectories;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Downloads authlib-injector from the master server so the launcher, wrapper
/// and node all run the exact same version. Before this the launcher fetched
/// the jar straight from upstream, causing version mismatches whenever the
/// upstream released a new build.
pub async fn ensure_authlib_injector(
    client: &reqwest::Client,
    config: &LauncherConfig,
    dirs: &LauncherDirectories,
) -> Result<PathBuf> {
    let path = dirs.authlib_injector();
    if path.exists() {
        return Ok(path);
    }

    tracing::info!("downloading authlib-injector from the master");
    let url = format!(
        "{}/api/agent/authlib-injector.jar",
        config.master_url.trim_end_matches('/')
    );
    let bytes = client
        .get(&url)
        .send()
        .await
        .context("authlib-injector: request failed")?
        .error_for_status()
        .context("authlib-injector: bad status")?
        .bytes()
        .await
        .context("authlib-injector: reading body")?;

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, &bytes).await?;
    Ok(path)
}
