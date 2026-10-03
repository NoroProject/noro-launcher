//! Shutdown that waits for the subsystems.
//!
//! The main thread holds the [`QuitCoordinator`] and hands every subsystem a
//! [`QuitHandler`] via `fork()`. Its `quit()` fires the stop callback and then
//! blocks until each fork has checked in, so a fork that is never dropped hangs
//! the exit.

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
            forks: Arc::new(AtomicU32::new(1)), // the coordinator itself counts as one
            notify: Arc::new(Notify::new()),
            on_quit,
        }
    }

    pub fn fork(&self) -> QuitHandler {
        self.forks.fetch_add(1, Ordering::SeqCst);
        QuitHandler {
            forks: self.forks.clone(),
            notify: self.notify.clone(),
            done: false,
        }
    }

    pub async fn quit(self) {
        (self.on_quit)();
        if self.forks.fetch_sub(1, Ordering::SeqCst) == 1 {
            return;
        }
        self.notify.notified().await;
    }
}

/// Checks in exactly once: on `quit()`, or when dropped without it. A backend
/// that returned an error or panicked never called `quit()`, and the
/// coordinator then waited forever — the process hung on exit, still holding
/// the single-instance lock, so the next launch only "focused" a dead window.
pub struct QuitHandler {
    forks: Arc<AtomicU32>,
    notify: Arc<Notify>,
    done: bool,
}

impl QuitHandler {
    pub fn quit(mut self) {
        self.check_in();
    }

    fn check_in(&mut self) {
        if std::mem::replace(&mut self.done, true) {
            return;
        }
        if self.forks.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.notify.notify_one();
        }
    }
}

/// A clone is one more subsystem to wait for, not a second key to the same one.
impl Clone for QuitHandler {
    fn clone(&self) -> Self {
        self.forks.fetch_add(1, Ordering::SeqCst);
        QuitHandler {
            forks: self.forks.clone(),
            notify: self.notify.clone(),
            done: false,
        }
    }
}

impl Drop for QuitHandler {
    fn drop(&mut self) {
        self.check_in();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn coordinator() -> QuitCoordinator {
        QuitCoordinator::new(Box::new(|| {}))
    }

    #[tokio::test]
    async fn a_dropped_fork_does_not_hang_the_exit() {
        let c = coordinator();
        let fork = c.fork();
        drop(fork);
        tokio::time::timeout(Duration::from_secs(1), c.quit())
            .await
            .expect("quit returned");
    }

    #[tokio::test]
    async fn quit_then_drop_counts_once() {
        let c = coordinator();
        let a = c.fork();
        let b = c.fork();
        a.quit();
        // `b` still running: quit must wait for it.
        let waiting = tokio::time::timeout(Duration::from_millis(50), c.quit());
        assert!(waiting.await.is_err());
        drop(b);
    }

    #[tokio::test]
    async fn a_clone_is_waited_for_too() {
        let c = coordinator();
        let a = c.fork();
        let a2 = a.clone();
        drop(a);
        let quit = tokio::spawn(c.quit());
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!quit.is_finished());
        drop(a2);
        tokio::time::timeout(Duration::from_secs(1), quit)
            .await
            .expect("quit returned")
            .unwrap();
    }
}
