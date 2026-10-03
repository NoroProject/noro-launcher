// Over 150 lines: the start-up sequence, then the splash preview used while
// working on the window.
//! Bootstrapper: check for a newer build, fetch the core binary, hand over.
//!
//! This binary is never updated after the first install, which is what lets it
//! accumulate SmartScreen reputation on Windows. Everything that looks like a
//! launcher lives in `noro-launcher-core`, which does update itself, under
//! `AppData/noro-launcher/`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod download;
mod embedded_config;
mod failure;
mod handover;
mod install;
mod log;
mod net;
mod splash;
mod verify;

use download::{download_core, update_pending};
use failure::short;
use handover::{core_binary_name, run_core, start_core};
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
            (
                download::downloading_label("launcher-v1.2.3", None),
                15_358_608,
            ),
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
                message: i18n::t(failure::Kind::Network.key()),
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
