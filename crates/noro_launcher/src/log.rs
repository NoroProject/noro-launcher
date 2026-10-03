//! A plain log file next to core's own.
//!
//! The release build has no console on Windows, so `eprintln!` went nowhere:
//! a player whose launcher never opened had nothing to send. "Report a
//! problem" in core picks this file up together with `launcher.log`.

use parking_lot::Mutex;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Past this the log starts over; the bootstrapper says little per run.
const MAX_BYTES: u64 = 256 * 1024;

static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn path(app_dir: &Path) -> PathBuf {
    app_dir.join("logs").join("bootstrapper.log")
}

pub fn init(app_dir: &Path) {
    let path = path(app_dir);
    let Some(dir) = path.parent() else {
        return;
    };
    let _ = std::fs::create_dir_all(dir);
    let too_big = std::fs::metadata(&path).is_ok_and(|m| m.len() > MAX_BYTES);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(!too_big)
        .truncate(too_big)
        .open(&path)
        .ok();
    *FILE.lock() = file;
    line(&format!(
        "noro-launcher {} starting on {}-{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
}

/// Also printed, for whoever runs it from a terminal.
pub fn line(msg: &str) {
    eprintln!("{msg}");
    if let Some(file) = FILE.lock().as_mut() {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(file, "[{secs}] {msg}");
    }
}
