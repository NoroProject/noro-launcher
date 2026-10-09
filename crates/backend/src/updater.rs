// Over 150 lines: the install, guarded against a second click, and the restart
// that follows it.
//! Launcher self-update: download, check sha256 and the ed25519 signature,
//! install into the data root.
//!
//! The bootstrapper — the .exe the user actually downloaded — is never
//! replaced. That's what lets it build up SmartScreen reputation on Windows.

use crate::directories::LauncherDirectories;
use crate::sync::integrity::sha256_hex;
use anyhow::{bail, Context, Result};
use schema::LauncherVersion;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

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

/// One install at a time. Two clicks used to start two downloads writing the
/// same binary.
static INSTALLING: AtomicBool = AtomicBool::new(false);

struct InstallGuard;

impl Drop for InstallGuard {
    fn drop(&mut self) {
        INSTALLING.store(false, Ordering::SeqCst);
    }
}

/// The signature file the bootstrapper checks on every start. It must describe
/// the binary next to it, or the bootstrapper throws the update away and
/// downloads it again — and offline, there is then nothing to start.
fn sig_path(core: &Path) -> PathBuf {
    let mut name = core.as_os_str().to_os_string();
    name.push(".sig");
    PathBuf::from(name)
}

/// Downloads and installs an update, returning the path to the new binary.
///
/// The new binary is written next to the old one and renamed over it. Writing
/// in place fails on Linux while the old one runs (`ETXTBSY`) and on macOS can
/// get the running process killed; a rename leaves the running image alone.
pub async fn install_update(
    client: &reqwest::Client,
    dirs: &LauncherDirectories,
    version: &LauncherVersion,
    on_progress: impl Fn(u64, u64),
) -> Result<PathBuf> {
    if INSTALLING.swap(true, Ordering::SeqCst) {
        bail!("an update is already being installed");
    }
    let _guard = InstallGuard;
    tokio::fs::create_dir_all(dirs.root()).await.ok();

    let resp = client.get(&version.url).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let mut bytes = Vec::with_capacity(total as usize);
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    let mut reported = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        bytes.extend_from_slice(&chunk);
        // Once per percent: the window doesn't need every packet.
        let done = bytes.len() as u64;
        let percent = (done * 100).checked_div(total);
        if percent.is_none_or(|p| p > reported) {
            reported = percent.unwrap_or(0);
            on_progress(done, total);
        }
    }

    // Nothing touches disk until both checks pass.
    let actual = sha256_hex(&bytes);
    if !actual.eq_ignore_ascii_case(&version.sha256) {
        bail!("sha256 mismatch: expected {}, got {actual}", version.sha256);
    }
    if !crate::signing::verify_bytes(&bytes, &version.signature) {
        bail!("binary signature is not valid");
    }

    let dest = core_binary_path(dirs);
    let mut staged = dest.clone().into_os_string();
    staged.push(".new");
    let staged = PathBuf::from(staged);
    crate::fsutil::write_atomic(&staged, &bytes)
        .await
        .with_context(|| format!("writing {}", staged.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&staged).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&staged, perms).await?;
    }

    // A running exe can't be replaced on Windows, but it can be renamed.
    #[cfg(windows)]
    if dest.exists() {
        let old = dest.with_extension("old");
        let _ = tokio::fs::remove_file(&old).await;
        tokio::fs::rename(&dest, &old)
            .await
            .context("moving the running launcher aside")?;
    }

    tokio::fs::rename(&staged, &dest)
        .await
        .with_context(|| format!("installing {}", dest.display()))?;
    crate::fsutil::write_atomic(sig_path(&dest), version.signature.trim())
        .await
        .context("writing the signature file")?;

    // This file is what decides whether an update is needed, so a failed write
    // is fatal rather than best-effort: without it every launch believes it's
    // out of date and fetches the same update again, silently and forever.
    let version_file = dirs.root().join("version");
    crate::fsutil::write_atomic(&version_file, &version.version)
        .await
        .with_context(|| format!("writing {}", version_file.display()))?;

    Ok(dest)
}

/// Set on the new process so it waits for this one's single-instance lock
/// instead of finding it taken and quitting.
pub const RESTART_ENV: &str = "NORO_RESTARTED";

/// Start the new binary and leave. If it can't be started, staying alive beats
/// leaving the player with nothing running.
pub fn restart(exe: &std::path::Path) {
    match std::process::Command::new(exe)
        .env(RESTART_ENV, "1")
        .spawn()
    {
        Ok(_) => std::process::exit(0),
        Err(e) => tracing::error!("could not start {}: {e}", exe.display()),
    }
}

#[cfg(test)]
#[path = "updater_tests.rs"]
mod tests;
