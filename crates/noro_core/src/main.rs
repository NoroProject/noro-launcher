//! The launcher itself: single-instance guard, then the frontend/backend pair.
//! This binary is the one the bootstrapper replaces on update.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bridge::{MessageToBackend, MessageToFrontend, QuitCoordinator};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::path::PathBuf;

fn main() {
    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(schema::launcher_dir_name());
    let _ = std::fs::create_dir_all(&app_dir);

    // The config path comes from backend rather than being rebuilt here, or the
    // player's setting and the file the launcher reads eventually drift apart.
    let config = backend::persistent::Persistent::<backend::config::LauncherConfig>::load(
        backend::LauncherDirectories::new().config_file(),
    );
    // Logging before telemetry, or everything telemetry says about its own
    // startup goes nowhere. The `error!` → event layer checks the hub itself, so
    // installing it before `init` is safe. Both go before anything that can
    // panic — the hook has to be in place first.
    backend::telemetry::init_tracing(backend::telemetry::is_enabled(&config.get()));
    let _sentry = backend::telemetry::init(&config.get());

    let lockfile_path = app_dir.join("app.lock");
    let socket_path = app_dir.join("app.sock");

    let lockfile = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        // The file is the lock; its contents aren't ours to clear.
        .truncate(false)
        .open(&lockfile_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("could not open app.lock: {e}");
            return;
        }
    };

    if lockfile.try_lock_exclusive().is_err() {
        focus_existing(&socket_path);
        return;
    }

    run_primary(&app_dir, &socket_path, &lockfile_path);
}

fn run_primary(
    _app_dir: &std::path::Path,
    socket_path: &std::path::Path,
    lockfile_path: &std::path::Path,
) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(3)
        .enable_all()
        .build()
        .expect("tokio runtime");

    let _ = std::fs::remove_file(socket_path);

    let (backend_recv, backend_handle, frontend_recv, frontend_handle) = bridge::create_pair();
    let listen_cancel = tokio_util::sync::CancellationToken::new();

    spawn_focus_listener(
        &runtime,
        socket_path.to_path_buf(),
        frontend_handle.clone(),
        listen_cancel.clone(),
    );

    let quit_coordinator = QuitCoordinator::new(Box::new({
        let backend_handle = backend_handle.clone();
        let listen_cancel = listen_cancel.clone();
        move || {
            listen_cancel.cancel();
            backend_handle.send(MessageToBackend::Quit);
        }
    }));

    backend::start(
        &runtime,
        frontend_handle,
        backend_recv,
        quit_coordinator.fork(),
    );

    // Blocks the main thread until the window closes — GPUI insists on running
    // there.
    frontend::start(backend_handle, frontend_recv);

    tracing::info!("frontend is gone, stopping the backend");
    runtime.block_on(quit_coordinator.quit());
    let _ = std::fs::remove_file(lockfile_path);
    // `exit` skips destructors, so the sentry guard never flushes on its own.
    backend::telemetry::flush();
    std::process::exit(0);
}

fn spawn_focus_listener(
    runtime: &tokio::runtime::Runtime,
    socket_path: PathBuf,
    frontend: bridge::FrontendHandle,
    cancel: tokio_util::sync::CancellationToken,
) {
    runtime.spawn(async move {
        #[cfg(unix)]
        {
            let listener = match tokio::net::UnixListener::bind(&socket_path) {
                Ok(l) => l,
                Err(e) => {
                    tracing::warn!("single-instance socket did not open: {e}");
                    return;
                }
            };
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        if accepted.is_ok() {
                            frontend.send(MessageToFrontend::OpenOrFocusMainWindow);
                        }
                    }
                    _ = cancel.cancelled() => break,
                }
            }
            let _ = std::fs::remove_file(&socket_path);
        }
        #[cfg(windows)]
        {
            use tokio::net::windows::named_pipe::ServerOptions;
            let pipe_name = r"\\.\pipe\noro-launcher";
            loop {
                let server = match ServerOptions::new().create(pipe_name) {
                    Ok(s) => s,
                    Err(_) => break,
                };
                tokio::select! {
                    res = server.connect() => {
                        if res.is_ok() {
                            frontend.send(MessageToFrontend::OpenOrFocusMainWindow);
                        }
                    }
                    _ = cancel.cancelled() => break,
                }
            }
        }
    });
}

/// The second process: poke the first one and let it raise its window.
fn focus_existing(socket_path: &std::path::Path) {
    println!("noro-launcher is already running, focusing its window");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    rt.block_on(async {
        #[cfg(unix)]
        {
            let _ = tokio::net::UnixStream::connect(socket_path).await;
        }
        #[cfg(windows)]
        {
            let _ = socket_path; // the pipe name is fixed
            use tokio::net::windows::named_pipe::ClientOptions;
            let _ = ClientOptions::new().open(r"\\.\pipe\noro-launcher");
        }
    });
}
