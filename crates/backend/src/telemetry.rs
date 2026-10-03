//! Crash reporting.
//!
//! The DSN is baked in at build time (`NORO_SENTRY_DSN`); without one the SDK
//! never starts and nothing leaves the machine. There is deliberately no default
//! DSN — these are players' machines, not our servers. Reporting can also be
//! turned off in settings, and no PII is attached either way.

use crate::config::LauncherConfig;
use sentry::SessionMode;

/// Held for the lifetime of the process; flushes the queue on drop.
pub type Guard = Option<sentry::ClientInitGuard>;

fn dsn() -> Option<&'static str> {
    option_env!("NORO_SENTRY_DSN").filter(|d| !d.trim().is_empty())
}

/// Prefers the tag from the build workflow. The Cargo version lags behind it, so
/// crashes would otherwise be filed under the previous release.
fn release() -> Option<std::borrow::Cow<'static, str>> {
    match option_env!("NORO_RELEASE").filter(|r| !r.trim().is_empty()) {
        Some(tag) => Some(tag.into()),
        None => sentry::release_name!(),
    }
}

/// Call before the runtime and GPUI come up — the panic hook needs to be in
/// place before anything has a chance to panic.
pub fn init(config: &LauncherConfig) -> Guard {
    let dsn = dsn()?;
    if !config.crash_reports {
        tracing::info!(target: "telemetry", "crash reports disabled by player");
        return None;
    }

    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: release(),
            environment: Some(if cfg!(debug_assertions) {
                "development".into()
            } else {
                "production".into()
            }),
            // Sentry fills this in from the hostname, which on a personal
            // machine is usually someone's name.
            server_name: None,
            send_default_pii: false,
            traces_sample_rate: 0.0,
            attach_stacktrace: true,
            auto_session_tracking: true,
            session_mode: SessionMode::Application,
            ..Default::default()
        },
    ));
    tracing::info!(target: "telemetry", release = ?release(), "crash reports enabled");
    Some(guard)
}

/// The launcher exits through `std::process::exit`, which skips destructors —
/// without this the guard never gets to send what it has queued.
pub fn flush() {
    sentry::end_session();
    if let Some(client) = sentry::Hub::current().client() {
        client.flush(Some(std::time::Duration::from_secs(2)));
    }
}

/// Launcher logs, plus `error` events into Sentry when it is running.
///
/// Most failures never panic: a download dies, a manifest signature doesn't
/// match, the game refuses to start. Without this layer none of it would show up.
pub fn init_tracing(sentry_on: bool) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,backend=debug,frontend=debug,bridge=debug".into());
    // A release build on Windows has no console: stdout goes nowhere, and every
    // warning the launcher wrote was lost — including from "report a problem".
    let file = open_log_file().map(|file| {
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(std::sync::Mutex::new(file))
    });
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(file);

    if sentry_on {
        registry.with(sentry_tracing::layer()).init();
    } else {
        registry.init();
    }
}

/// Whether a DSN is baked into this build. Without one the settings toggle is a
/// lie — it flips, but there is nowhere to send.
pub fn is_available() -> bool {
    dsn().is_some()
}

/// Needed before `init`, because the log subscriber is installed first.
pub fn is_enabled(config: &LauncherConfig) -> bool {
    is_available() && config.crash_reports
}

/// Past this the log is rotated at startup, keeping one previous file.
const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

/// Where the launcher's own log lives; "report a problem" picks it up there.
pub fn log_path() -> std::path::PathBuf {
    crate::LauncherDirectories::new()
        .root()
        .join("logs")
        .join("launcher.log")
}

fn open_log_file() -> Option<std::fs::File> {
    let path = log_path();
    std::fs::create_dir_all(path.parent()?).ok()?;
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > MAX_LOG_BYTES) {
        let _ = std::fs::rename(&path, path.with_extension("log.1"));
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()
}
