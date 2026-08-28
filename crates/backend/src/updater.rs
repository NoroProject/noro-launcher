//! Launcher self-update: download, check sha256 and the ed25519 signature,
//! install into the data root.
//!
//! The bootstrapper — the .exe the user actually downloaded — is never
//! replaced. That's what lets it build up SmartScreen reputation on Windows.

use crate::directories::LauncherDirectories;
use crate::sync::integrity::sha256_hex;
use anyhow::{bail, Context, Result};
use schema::LauncherVersion;
use std::path::PathBuf;

fn core_binary_name() -> &'static str {
    if cfg!(windows) {
        "noro-launcher-core.exe"
    } else {
        "noro-launcher-core"
    }
}

/// The binary the bootstrapper launches, as opposed to the bootstrapper itself.
pub fn core_binary_path(dirs: &LauncherDirectories) -> PathBuf {
    dirs.root().join(core_binary_name())
}

/// Downloads and installs an update, returning the path to the new binary.
pub async fn install_update(
    client: &reqwest::Client,
    dirs: &LauncherDirectories,
    version: &LauncherVersion,
    on_progress: impl Fn(u64, u64),
) -> Result<PathBuf> {
    tokio::fs::create_dir_all(dirs.root()).await.ok();

    let resp = client.get(&version.url).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let mut bytes = Vec::with_capacity(total as usize);
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        bytes.extend_from_slice(&chunk);
        on_progress(bytes.len() as u64, total);
    }

    // Nothing touches disk until both checks pass.
    let actual = sha256_hex(&bytes);
    if !actual.eq_ignore_ascii_case(&version.sha256) {
        bail!(
            "sha256 mismatch: expected {}, got {actual}",
            version.sha256
        );
    }
    if !crate::signing::verify_bytes(&bytes, &version.signature) {
        bail!("binary signature is not valid");
    }

    let dest = core_binary_path(dirs);

    // A running exe can't be overwritten on Windows, so move the old one aside.
    #[cfg(windows)]
    if dest.exists() {
        let old = dest.with_extension("old");
        let _ = tokio::fs::remove_file(&old).await;
        let _ = tokio::fs::rename(&dest, &old).await;
    }

    tokio::fs::write(&dest, &bytes).await?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&dest).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&dest, perms).await?;
    }

    // This file is what decides whether an update is needed, so a failed write
    // is fatal rather than best-effort: without it every launch believes it's
    // out of date and fetches the same update again, silently and forever.
    let version_file = dirs.root().join("version");
    tokio::fs::write(&version_file, &version.version)
        .await
        .with_context(|| format!("writing {}", version_file.display()))?;

    Ok(dest)
}

pub fn restart(exe: &std::path::Path) -> ! {
    let _ = std::process::Command::new(exe).spawn();
    std::process::exit(0);
}
