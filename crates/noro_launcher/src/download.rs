// Over 150 lines: the update check and the download share the version URL and
// the platform name, and the download is one sequence that reads top to bottom.
//! Asking the master for the current core and fetching it.

use crate::failure::{Failure, Kind};
use crate::{install, log, net, splash, verify};
use i18n::FluentArgs;
use std::path::Path;

/// Is the master serving a different version than the one installed?
///
/// Updating core is the bootstrapper's job. Core can't do it itself: its update
/// button lives in settings, settings live behind the login screen, and when the
/// update is what login needs, that circle never opens.
///
/// No network, or a silent master, means launching what we have. Getting into
/// the game matters more than being current. No window is up yet, so the
/// wait is short.
pub async fn update_pending(app_dir: &Path) -> bool {
    let installed = std::fs::read_to_string(app_dir.join("version")).unwrap_or_default();
    let installed = installed.trim();
    if installed.is_empty() {
        return false;
    }
    let resp = net::quick_client().get(version_url()).send().await;
    let Ok(resp) = resp.and_then(|r| r.error_for_status()) else {
        return false;
    };
    let Ok(info) = resp.json::<serde_json::Value>().await else {
        return false;
    };
    info["version"]
        .as_str()
        .is_some_and(|remote| remote != installed)
}

fn version_url() -> String {
    format!(
        "{}/api/launcher/version?platform={}",
        verify::master_url().trim_end_matches('/'),
        current_platform()
    )
}

pub fn downloading_label(version: &str, size: Option<u64>) -> String {
    let mut args = FluentArgs::new();
    args.set("version", version.to_string());
    match size {
        Some(bytes) => {
            args.set("size", format!("{:.1}", bytes as f64 / 1_048_576.0));
            i18n::t_args("boot-downloading-size", &args)
        }
        None => i18n::t_args("boot-downloading", &args),
    }
}

pub async fn download_core(
    app_dir: &Path,
    dest: &Path,
    report: &splash::Reporter,
) -> Result<(), Failure> {
    let say = |label: String, done: u64, total: u64| {
        let _ = report.send(splash::Update::Progress(splash::Progress {
            label,
            done,
            total,
        }));
    };
    say(i18n::t("boot-checking"), 0, 0);
    let platform = current_platform();
    let client = net::client();
    let url = version_url();
    let info: serde_json::Value = net::retry(|| async {
        client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    })
    .await
    .map_err(Failure::http)?;

    let unavailable = |what: &str| Failure::new(Kind::Unavailable, anyhow::anyhow!("{what}"));
    if info.is_null() {
        return Err(unavailable(&format!("no launcher build for {platform}")));
    }
    let download_url = info["url"]
        .as_str()
        .ok_or_else(|| unavailable("no url in the response"))?;
    // Both of these are required rather than optional. The sha256 catches a
    // corrupted download but not a substituted one — whoever can swap the file
    // can swap the hash beside it — so it's the signature that decides, and an
    // absent one must fail here rather than quietly skip the check below.
    let expected_sha = info["sha256"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| unavailable("master sent no sha256"))?;
    let signature = info["signature"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| unavailable("master sent no signature"))?;
    // This ends up in the version file next to the binary, so a placeholder
    // would leave the next update check comparing against nonsense.
    let version = info["version"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| unavailable("master sent no version"))?;

    let bytes = net::retry(|| async {
        say(downloading_label(version, None), 0, 0);
        let mut resp = client.get(download_url).send().await?.error_for_status()?;
        // Chunk by chunk for the progress bar; reqwest hands them over without
        // dragging in futures.
        let total = resp.content_length().unwrap_or(0);
        let mut bytes: Vec<u8> = Vec::with_capacity(total as usize);
        // One report per percent, or per half megabyte when the size isn't
        // known. The window redraws once either way, and reporting per chunk
        // would be a message for every packet.
        let mut reported = 0u64;
        while let Some(chunk) = resp.chunk().await? {
            bytes.extend_from_slice(&chunk);
            let done = bytes.len() as u64;
            match (done * 100).checked_div(total) {
                Some(percent) if percent > reported => {
                    reported = percent;
                    say(downloading_label(version, None), done, total);
                }
                Some(_) => {}
                None if done / (512 * 1024) > reported => {
                    reported = done / (512 * 1024);
                    say(downloading_label(version, Some(done)), 0, 0);
                }
                None => {}
            }
        }
        Ok(bytes)
    })
    .await
    .map_err(Failure::http)?;

    use sha2::Digest;
    let hash = hex::encode(sha2::Sha256::digest(&bytes));
    if !hash.eq_ignore_ascii_case(expected_sha) {
        return Err(Failure::new(
            Kind::Corrupt,
            anyhow::anyhow!("sha256 mismatch: expected {expected_sha}, got {hash}"),
        ));
    }
    verify::verify_bytes(&bytes, signature)
        .map_err(|e| Failure::new(Kind::Corrupt, e.context("launcher signature check failed")))?;

    install::install(app_dir, dest, &bytes, signature, version).map_err(|e| {
        Failure::new(
            Kind::Disk,
            anyhow::Error::new(e).context(format!("could not install into {}", app_dir.display())),
        )
    })?;
    log::line(&format!("installed {version}"));

    say(i18n::t("boot-starting"), 1, 1);
    Ok(())
}

fn current_platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        ("windows", "x86_64") => "windows-x86_64",
        (os, arch) => Box::leak(format!("{os}-{arch}").into_boxed_str()),
    }
}
