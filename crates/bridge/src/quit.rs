//! Координация корректного завершения. Главный поток создаёт [`QuitCoordinator`]
//! с колбэком остановки; каждая подсистема получает [`QuitHandler`] через `fork()`.
//! `quit()` запускает остановку и ждёт, пока все форки отметятся.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

pub struct QuitCoordinator {
    forks: Arc<AtomicU32>,
    notify: Arc<Notify>,
    on_quit: Box<dyn Fn() + Send + Sync>,
}

impl QuitCoordinator {
    pub fn new(on_quit: Box<dyn Fn() + Send + Sync>) -> Self {
        Self {
            forks: Arc::new(AtomicU32::new(1)), // 1 — за сам координатор
            notify: Arc::new(Notify::new()),
            on_quit,
        }
    }

    pub fn fork(&self) -> QuitHandler {
        self.forks.fetch_add(1, Ordering::SeqCst);
        QuitHandler {
            forks: self.forks.clone(),
            notify: self.notify.clone(),
        }
    }

    /// Запустить остановку и дождаться, пока все форки вызовут quit().
    pub async fn quit(self) {
        (self.on_quit)();
        if self.forks.fetch_sub(1, Ordering::SeqCst) == 1 {
            return;
        }
        self.notify.notified().await;
    }
}

#[derive(Clone)]
pub struct QuitHandler {
    forks: Arc<AtomicU32>,
    notify: Arc<Notify>,
}

impl QuitHandler {
    pub fn quit(self) {
        if self.forks.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.notify.notify_one();
        }
    }
}
