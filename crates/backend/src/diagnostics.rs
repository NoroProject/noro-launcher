//! Diagnostic snapshot of the launcher.
//!
//! Versions, hardware, latency to the master. Nothing personal goes in here,
//! which is why it is gathered without asking first.

use crate::directories::LauncherDirectories;
use schema::DiagnosticsReport;
use std::path::Path;
use std::time::Instant;

pub async fn collect(
    http: &reqwest::Client,
    dirs: &LauncherDirectories,
    master_url: &str,
    last_sync_error: Option<String>,
) -> DiagnosticsReport {
    DiagnosticsReport {
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        java_path: find_java(dirs),
        disk_free_mb: free_space_mb(dirs.root()),
        data_size_mb: dir_size_mb(dirs.root()).await,
        master_ping_ms: ping(http, master_url).await,
        last_sync_error,
        instances: instances(dirs).await,
    }
}

/// Takes the runtime from whichever instance has one. They're interchangeable;
/// the only question being answered is whether Java is there at all.
fn find_java(dirs: &LauncherDirectories) -> Option<String> {
    let instances = dirs.instances();
    let entries = std::fs::read_dir(instances).ok()?;
    for e in entries.flatten() {
        for rel in ["runtime/bin/java", "runtime/bin/java.exe"] {
            let candidate = e.path().join(rel);
            if candidate.exists() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Free space on the volume holding the instances. A download that stopped
/// because the disk filled up leaves no trace in any game log.
fn free_space_mb(path: &Path) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let Ok(c_path) = std::ffi::CString::new(path.as_os_str().as_bytes()) else {
            return 0;
        };
        // SAFETY: valid C string in, statvfs fills the struct completely.
        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
                return 0;
            }
            // `f_bavail` is 32-bit on macOS and 64-bit on Linux, so the cast is
            // required on one and redundant on the other. Silencing the lint is
            // the only way to keep both platforms compiling.
            #[allow(clippy::unnecessary_cast)]
            let free = (stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64);
            free / 1_048_576
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        0
    }
}

async fn dir_size_mb(root: &Path) -> u64 {
    let root = root.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let bytes: u64 = walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum();
        bytes / 1_048_576
    })
    .await
    .unwrap_or(0)
}

async fn ping(http: &reqwest::Client, master_url: &str) -> Option<u64> {
    let url = format!("{}/health", master_url.trim_end_matches('/'));
    let started = Instant::now();
    http.get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .ok()
        .map(|_| started.elapsed().as_millis() as u64)
}

/// Instance id paired with the build recorded in its `.noro-build`.
async fn instances(dirs: &LauncherDirectories) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(dirs.instances()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let version = tokio::fs::read_to_string(e.path().join(".noro-build"))
            .await
            .unwrap_or_else(|_| "?".into())
            .trim()
            .to_string();
        out.push((name, version));
    }
    out
}
