// File exceeds 150 lines: bootstrapper lifecycle managing initial download, signature check, and core handoff.
//! Bootstrapper: check for a newer build, fetch the core binary, hand over.
//!
//! This binary is never updated after the first install, which is what lets it
//! accumulate SmartScreen reputation on Windows. Everything that looks like a
//! launcher lives in `noro-launcher-core`, which does update itself, under
//! `AppData/noro-launcher/`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod embedded_config;
mod install;
mod log;
mod net;
mod splash;
mod verify;

use i18n::FluentArgs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    // Look at the loading window without waiting for a real download. The only
    // other way to work on it is deleting the installed core before every run.
    #[cfg(debug_assertions)]
    if std::env::var_os("NORO_SPLASH_PREVIEW").is_some() {
        set_locale(Path::new("."));
        return splash_preview();
    }

    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(schema::launcher_dir_name());
    let _ = std::fs::create_dir_all(&app_dir);
    log::init(&app_dir);
    set_locale(&app_dir);

    let core_path = app_dir.join(core_binary_name());
    // Left by an update while the previous core was running. It isn't running
    // any more by the time this can delete it, or the delete fails quietly.
    let _ = std::fs::remove_file(install::moved_aside(&core_path));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    // The signature is checked on every launch, not only after a download:
    // otherwise anything that can write to AppData gets executed forever.
    let installed = core_path.exists()
        && match verify::verify_installed(&core_path) {
            Ok(()) => true,
            Err(e) => {
                log::line(&format!(
                    "the installed launcher failed its signature check: {e:#}; downloading it again"
                ));
                verify::discard(&core_path);
                false
            }
        };
    if installed {
        if !rt.block_on(update_pending(&app_dir)) {
            return run_core(&core_path, &app_dir);
        }
        log::line("master has a different version, updating");
    }

    // First run, a rejected core, or an update. What follows can take minutes,
    // so the window goes up: GPUI takes the main thread, the download runs
    // behind it and reports progress through the channel.
    let (reporter, rx) = tokio::sync::mpsc::unbounded_channel();
    let outcome = splash::run_with(rx, move |choices| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        install_and_start(&rt, &app_dir, &core_path, installed, &reporter, &choices)
    });

    // Linux only: on macOS and Windows GPUI ends the process inside
    // `run_with`. Core has either been started by now or the player gave up.
    match outcome {
        Some(true) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

/// Runs behind the splash until core is running or the player closes the
/// window. Core is started from in here rather than after `run_with`,
/// because only Linux ever reaches the code after it.
fn install_and_start(
    rt: &tokio::runtime::Runtime,
    app_dir: &Path,
    core: &Path,
    installed: bool,
    report: &splash::Reporter,
    choices: &splash::Choices,
) -> bool {
    let mut downloaded = false;
    loop {
        let failure = if downloaded {
            None
        } else {
            match rt.block_on(download_core(app_dir, core, report)) {
                Ok(()) => {
                    downloaded = true;
                    None
                }
                Err(f) => Some(f),
            }
        };
        let failure = match failure {
            None => start_core(core, app_dir).err(),
            // No network, or a silent master: launch what we have. Getting
            // into the game matters more than being current.
            Some(f) if installed && verify::verify_installed(core).is_ok() => {
                log::line(&format!(
                    "update failed ({:#}), starting the installed version",
                    f.error
                ));
                start_core(core, app_dir).err()
            }
            Some(f) => Some(f),
        };
        let Some(failure) = failure else {
            return true;
        };

        log::line(&format!("{:#}", failure.error));
        let _ = report.send(splash::Update::Failed {
            message: i18n::t(failure.kind.key()),
            detail: short(&format!("{:#}", failure.error)),
        });
        match choices.recv() {
            Ok(splash::Choice::Retry) => log::line("retrying"),
            _ => return false,
        }
    }
}

/// One line that fits under the message.
fn short(detail: &str) -> String {
    const MAX: usize = 140;
    let line = detail.lines().next().unwrap_or_default();
    if line.chars().count() <= MAX {
        line.to_string()
    } else {
        let cut: String = line.chars().take(MAX).collect();
        format!("{cut}…")
    }
}

/// What went wrong, in terms of what the player can do about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Couldn't talk to the master: check the connection, try again.
    Network,
    /// The master answered but has nothing usable for this system.
    Unavailable,
    /// The download doesn't match its hash or signature.
    Corrupt,
    /// Writing the files failed: a full disk, permissions, an antivirus.
    Disk,
    /// Core is in place but wouldn't start.
    Start,
}

impl Kind {
    fn key(self) -> &'static str {
        match self {
            Kind::Network => "boot-error-network",
            Kind::Unavailable => "boot-error-unavailable",
            Kind::Corrupt => "boot-error-corrupt",
            Kind::Disk => "boot-error-disk",
            Kind::Start => "boot-error-start",
        }
    }
}

struct Failure {
    kind: Kind,
    error: anyhow::Error,
}

impl Failure {
    fn new(kind: Kind, error: impl Into<anyhow::Error>) -> Self {
        Self {
            kind,
            error: error.into(),
        }
    }

    /// A 4xx or an unreadable answer won't change on retry; the rest is the
    /// link.
    fn http(e: reqwest::Error) -> Self {
        let kind = if e.is_decode() || e.status().is_some_and(|s| s.is_client_error()) {
            Kind::Unavailable
        } else {
            Kind::Network
        };
        Self::new(kind, e)
    }
}

/// The player's choice from core's settings, or the system language on a
/// first run.
fn set_locale(app_dir: &Path) {
    let chosen = std::fs::read_to_string(app_dir.join("config.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|config| config["locale"].as_str().map(str::to_string));
    let code = chosen.or_else(sys_locale::get_locale).unwrap_or_default();
    i18n::set_locale(i18n::Locale::from_code(&code).unwrap_or_default());
}

/// Runs the bar around in circles, failing every other pass, until the window
/// is closed.
#[cfg(debug_assertions)]
fn splash_preview() -> ExitCode {
    let (reporter, rx) = tokio::sync::mpsc::unbounded_channel();
    splash::run_with(rx, move |choices| {
        let stages = [
            (i18n::t("boot-checking"), 0u64),
            (downloading_label("launcher-v1.2.3", None), 15_358_608),
        ];
        loop {
            for (label, total) in &stages {
                for step in 0..=100 {
                    let _ = reporter.send(splash::Update::Progress(splash::Progress {
                        label: label.clone(),
                        done: total / 100 * step,
                        total: *total,
                    }));
                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
            }
            let _ = reporter.send(splash::Update::Failed {
                message: i18n::t(Kind::Network.key()),
                detail: "error sending request for url (https://example.com/api/launcher/version)"
                    .into(),
            });
            if choices.recv() != Ok(splash::Choice::Retry) {
                return;
            }
        }
    });
    ExitCode::SUCCESS
}

fn core_binary_name() -> &'static str {
    if cfg!(windows) {
        "noro-launcher-core.exe"
    } else {
        "noro-launcher-core"
    }
}

fn prepare_core_cmd(path: &Path, app_dir: &Path) -> std::process::Command {
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

fn run_core(path: &Path, app_dir: &Path) -> ExitCode {
    let mut cmd = prepare_core_cmd(path, app_dir);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        log::line(&format!("could not start {}: {err}", path.display()));
        ExitCode::FAILURE
    }

    #[cfg(not(unix))]
    {
        match cmd.spawn() {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                log::line(&format!("could not start {}: {e}", path.display()));
                ExitCode::FAILURE
            }
        }
    }
}

/// From behind the splash. On Unix this replaces the process and only comes
/// back on failure; on Windows core is spawned and the bootstrapper exits immediately.
fn start_core(path: &Path, app_dir: &Path) -> Result<(), Failure> {
    let mut cmd = prepare_core_cmd(path, app_dir);
    #[cfg(unix)]
    let err = {
        use std::os::unix::process::CommandExt;
        cmd.exec()
    };
    #[cfg(not(unix))]
    {
        match cmd.spawn() {
            Ok(_) => std::process::exit(0),
            Err(err) => {
                return Err(Failure::new(
                    Kind::Start,
                    anyhow::Error::new(err).context(format!("could not start {}", path.display())),
                ));
            }
        }
    }
    #[cfg(unix)]
    Err(Failure::new(
        Kind::Start,
        anyhow::Error::new(err).context(format!("could not start {}", path.display())),
    ))
}

/// Is the master serving a different version than the one installed?
///
/// Updating core is the bootstrapper's job. Core can't do it itself: its update
/// button lives in settings, settings live behind the login screen, and when the
/// update is what login needs, that circle never opens.
///
/// No network, or a silent master, means launching what we have. Getting into
/// the game matters more than being current. No window is up yet, so the
/// wait is short.
async fn update_pending(app_dir: &Path) -> bool {
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

fn downloading_label(version: &str, size: Option<u64>) -> String {
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

async fn download_core(
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
