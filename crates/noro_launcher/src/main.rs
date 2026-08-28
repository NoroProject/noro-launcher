//! Bootstrapper: check for a newer build, fetch the core binary, hand over.
//!
//! This binary is never updated after the first install, which is what lets it
//! accumulate SmartScreen reputation on Windows. Everything that looks like a
//! launcher lives in `noro-launcher-core`, which does update itself, under
//! `AppData/noro-launcher/`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod embedded_config;
mod splash;
mod verify;

use anyhow::Context;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    // Look at the loading window without waiting for a real download. The only
    // other way to work on it is deleting the installed core before every run.
    #[cfg(debug_assertions)]
    if std::env::var_os("NORO_SPLASH_PREVIEW").is_some() {
        return splash_preview();
    }

    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(schema::launcher_dir_name());
    let _ = std::fs::create_dir_all(&app_dir);

    let core_path = app_dir.join(core_binary_name());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    // The signature is checked on every launch, not only after a download:
    // otherwise anything that can write to AppData gets executed forever.
    if core_path.exists() {
        match verify::verify_installed(&core_path) {
            Ok(()) if !rt.block_on(update_pending(&app_dir)) => return run_core(&core_path, &app_dir),
            Ok(()) => eprintln!("master has a different version, updating"),
            Err(e) => {
                eprintln!("the installed launcher failed its signature check: {e:#}");
                eprintln!("downloading it again");
                verify::discard(&core_path);
            }
        }
    }

    // First run, a rejected core, or an update. What follows can take minutes,
    // so the window goes up: GPUI takes the main thread, the download runs
    // behind it and reports progress through the channel.
    let (reporter, rx) = tokio::sync::mpsc::unbounded_channel();
    let work_dir = app_dir.clone();
    let work_core = core_path.clone();
    let launch_path = core_path.clone();
    let splash_app_dir = app_dir.clone();

    // Core is started from the `run_with` callback rather than after it,
    // because only Linux ever reaches the code after it: on macOS and Windows
    // GPUI takes the process down from inside `run_with`. Move this out and the
    // update ends with the window closing and nothing starting.
    let outcome = splash::run_with(
        rx,
        move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(download_core(&work_dir, &work_core, &reporter))
        },
        Some(Box::new(move |res: &anyhow::Result<()>| {
            if res.is_err() {
                return;
            }
            let mut cmd = prepare_core_cmd(&launch_path, &splash_app_dir);
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let err = cmd.exec();
                eprintln!("could not start {}: {err}", launch_path.display());
            }
            #[cfg(not(unix))]
            if let Err(e) = cmd.spawn() {
                eprintln!("could not start {}: {e}", launch_path.display());
            }
        })),
    );

    // Linux only, and by now the callback has already started core.
    match outcome {
        Some(Ok(())) => ExitCode::SUCCESS,
        Some(Err(e)) => {
            eprintln!("download failed: {e:#}");
            ExitCode::FAILURE
        }
        None => ExitCode::FAILURE,
    }
}

/// Runs the bar around in circles until the window is closed.
#[cfg(debug_assertions)]
fn splash_preview() -> ExitCode {
    let (reporter, rx) = tokio::sync::mpsc::unbounded_channel();
    splash::run_with(
        rx,
        move || {
            let stages = [
                ("Проверка версии…", 0u64),
                ("Загрузка launcher-v1.2.3", 15_358_608),
            ];
            loop {
                for (label, total) in stages {
                    for step in 0..=100 {
                        let _ = reporter.send(splash::Progress {
                            label: label.to_string(),
                            done: total / 100 * step,
                            total,
                        });
                        std::thread::sleep(std::time::Duration::from_millis(60));
                    }
                }
            }
        },
        None,
    );
    ExitCode::SUCCESS
}

fn core_binary_name() -> &'static str {
    if cfg!(windows) {
        "noro-launcher-core.exe"
    } else {
        "noro-launcher-core"
    }
}

fn prepare_core_cmd(path: &std::path::Path, app_dir: &std::path::Path) -> std::process::Command {
    let master_url = verify::master_url();
    let pubkey = verify::raw_signing_pubkey();

    let bootstrap_path = app_dir.join("bootstrap.json");
    let bootstrap_data = serde_json::json!({
        "master_url": master_url,
        "signing_pubkey": pubkey,
    });
    if let Ok(json_str) = serde_json::to_string_pretty(&bootstrap_data) {
        let _ = std::fs::write(bootstrap_path, json_str);
    }

    let mut cmd = std::process::Command::new(path);
    cmd.args(std::env::args_os().skip(1));
    cmd.env("NORO_MASTER_URL", &master_url);
    if !pubkey.is_empty() {
        cmd.env("NORO_SIGNING_PUBKEY", &pubkey);
    }
    cmd
}

fn run_core(path: &std::path::Path, app_dir: &std::path::Path) -> ExitCode {
    let mut cmd = prepare_core_cmd(path, app_dir);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        eprintln!("could not start {}: {err}", path.display());
        ExitCode::FAILURE
    }

    #[cfg(not(unix))]
    {
        match cmd.status() {
            Ok(s) => {
                if s.success() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(e) => {
                eprintln!("could not start {}: {e}", path.display());
                ExitCode::FAILURE
            }
        }
    }
}

/// Is the master serving a different version than the one installed?
///
/// Updating core is the bootstrapper's job. Core can't do it itself: its update
/// button lives in settings, settings live behind the login screen, and when the
/// update is what login needs, that circle never opens.
///
/// No network, or a silent master, means launching what we have. Getting into
/// the game matters more than being current.
async fn update_pending(app_dir: &Path) -> bool {
    let installed = std::fs::read_to_string(app_dir.join("version")).unwrap_or_default();
    let installed = installed.trim();
    if installed.is_empty() {
        return false;
    }
    let url = format!(
        "{}/api/launcher/version?platform={}",
        verify::master_url().trim_end_matches('/'),
        current_platform()
    );
    let Ok(resp) = reqwest::Client::new().get(&url).send().await else {
        return false;
    };
    let Ok(info) = resp.json::<serde_json::Value>().await else {
        return false;
    };
    info["version"]
        .as_str()
        .is_some_and(|remote| remote != installed)
}

async fn download_core(
    app_dir: &Path,
    dest: &Path,
    report: &splash::Reporter,
) -> anyhow::Result<()> {
    let say = |label: &str, done: u64, total: u64| {
        let _ = report.send(splash::Progress {
            label: label.to_string(),
            done,
            total,
        });
    };
    say("Проверка версии…", 0, 0);
    let master_url = verify::master_url();
    let platform = current_platform();
    let url = format!(
        "{}/api/launcher/version?platform={platform}",
        master_url.trim_end_matches('/')
    );

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?.error_for_status()?;
    let info: serde_json::Value = resp.json().await?;

    if info.is_null() {
        anyhow::bail!("no launcher build for {platform}");
    }

    let download_url = info["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no url in the response"))?;
    // Both of these are required rather than optional. The sha256 catches a
    // corrupted download but not a substituted one — whoever can swap the file
    // can swap the hash beside it — so it's the signature that decides, and an
    // absent one must fail here rather than quietly skip the check below.
    let expected_sha = info["sha256"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("master sent no sha256"))?;
    let signature = info["signature"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("master sent no signature"))?;
    // This ends up in the version file next to the binary, so a placeholder
    // would leave the next update check comparing against nonsense.
    let version = info["version"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("master sent no version"))?;

    say(&format!("Загрузка {version}"), 0, 0);
    let mut resp = client.get(download_url).send().await?.error_for_status()?;

    // Chunk by chunk for the progress bar; reqwest hands them over without
    // dragging in futures.
    let total = resp.content_length().unwrap_or(0);
    let mut bytes: Vec<u8> = Vec::with_capacity(total as usize);
    // One report per percent. The window redraws once either way, and reporting
    // per chunk would be a message for every packet.
    let mut reported = 0u64;
    while let Some(chunk) = resp.chunk().await? {
        bytes.extend_from_slice(&chunk);
        let done = bytes.len() as u64;
        if total > 0 && done * 100 / total > reported {
            reported = done * 100 / total;
            say(&format!("Загрузка {version}"), done, total);
        }
    }

    use sha2::Digest;
    let hash = hex::encode(sha2::Sha256::digest(&bytes));
    if !hash.eq_ignore_ascii_case(expected_sha) {
        anyhow::bail!("sha256 mismatch: expected {expected_sha}, got {hash}");
    }

    verify::verify_bytes(&bytes, signature)
        .map_err(|e| anyhow::anyhow!("launcher signature check failed: {e}"))?;

    std::fs::write(dest, &bytes)?;
    verify::store(dest, signature)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(dest, perms)?;
    }

    // Without this the launcher downloads itself again on every start.
    let version_file = app_dir.join("version");
    std::fs::write(&version_file, version)
        .with_context(|| format!("could not write {}", version_file.display()))?;

    say("Готово", 1, 1);
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
