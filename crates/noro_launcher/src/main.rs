//! Точка входа лаунчера: single-instance, мост frontend↔backend, запуск обоих.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bridge::{MessageToBackend, MessageToFrontend, QuitCoordinator};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::path::PathBuf;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend=debug,frontend=debug,bridge=debug".into()),
        )
        .init();

    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("noro-launcher");
    let _ = std::fs::create_dir_all(&app_dir);

    let lockfile_path = app_dir.join("app.lock");
    let socket_path = app_dir.join("app.sock");

    let lockfile = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lockfile_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("не удалось открыть app.lock: {e}");
            return;
        }
    };

    if lockfile.try_lock_exclusive().is_err() {
        // Уже запущен — попросить существующий процесс показать окно.
        focus_existing(&socket_path);
        return;
    }

    run_primary(&app_dir, &socket_path, &lockfile_path);
}

/// Основной процесс.
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

    // Слушатель single-instance: на новое подключение — показать окно.
    spawn_focus_listener(
        &runtime,
        socket_path.to_path_buf(),
        frontend_handle.clone(),
        listen_cancel.clone(),
    );

    // Координатор завершения: отменяет слушатель и шлёт Quit в backend.
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

    // Блокирует главный поток до выхода (GPUI требует main thread).
    frontend::start(backend_handle, frontend_recv);

    tracing::info!("frontend завершён, останавливаем backend");
    runtime.block_on(quit_coordinator.quit());
    let _ = std::fs::remove_file(lockfile_path);
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
                    tracing::warn!("не удалось открыть сокет single-instance: {e}");
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

/// Вторичный процесс: достучаться до основного, чтобы показать окно.
fn focus_existing(socket_path: &std::path::Path) {
    println!("noro-launcher уже запущен — показываю окно");
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
            let _ = socket_path; // имя пайпа фиксировано
            use tokio::net::windows::named_pipe::ClientOptions;
            let _ = ClientOptions::new().open(r"\\.\pipe\noro-launcher");
        }
    });
}
