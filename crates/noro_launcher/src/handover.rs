//! Starting core: the arguments it is handed and how the process is replaced.

use crate::failure::{Failure, Kind};
use crate::{log, verify};
use std::path::Path;
use std::process::ExitCode;

pub fn core_binary_name() -> &'static str {
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

pub fn run_core(path: &Path, app_dir: &Path) -> ExitCode {
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
        match cmd.status() {
            Ok(s) if s.success() => ExitCode::SUCCESS,
            Ok(_) => ExitCode::FAILURE,
            Err(e) => {
                log::line(&format!("could not start {}: {e}", path.display()));
                ExitCode::FAILURE
            }
        }
    }
}

/// From behind the splash. On Unix this replaces the process and only comes
/// back on failure; on Windows core is spawned and the splash then quits.
pub fn start_core(path: &Path, app_dir: &Path) -> Result<(), Failure> {
    let mut cmd = prepare_core_cmd(path, app_dir);
    #[cfg(unix)]
    let err = {
        use std::os::unix::process::CommandExt;
        cmd.exec()
    };
    #[cfg(not(unix))]
    let Err(err) = cmd.spawn().map(drop) else {
        return Ok(());
    };
    Err(Failure::new(
        Kind::Start,
        anyhow::Error::new(err).context(format!("could not start {}", path.display())),
    ))
}
