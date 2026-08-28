//! The loopback channel to the in-game moderation mod.
//!
//! The mod gets neither a token nor the admin URL: it sends an intent and the
//! launcher talks to the master itself. The listener comes up with the game and
//! dies with it, so the window a hostile local page could use stays short.

mod handshake;
mod intents;
#[cfg(test)]
mod live_tests;
mod master;
mod push;
mod server;
mod store;

pub use store::CaseStore;

use crate::backend::Ctx;
use mod_link::ToMod;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Default)]
struct Inner {
    /// The key from the handshake file. Empty while no listener is up.
    key: String,
    instance_dir: Option<PathBuf>,
    /// Where frames go while the mod is connected. One connection only — the
    /// moderator has one panel.
    tx: Option<UnboundedSender<ToMod>>,
    accept: Option<JoinHandle<()>>,
}

#[derive(Clone, Default)]
pub struct ModLink {
    inner: Arc<Mutex<Inner>>,
    store: CaseStore,
}

impl ModLink {
    pub fn store(&self) -> CaseStore {
        self.store.clone()
    }

    /// Bring up the listener and drop the handshake file in the instance
    /// directory.
    ///
    /// Every failure here is warn-and-continue: without the channel the mod
    /// won't find the launcher and goes without its panel, but the game still
    /// starts.
    pub async fn start(&self, ctx: &Ctx, instance_dir: PathBuf) {
        self.stop().await;
        let listener = match TcpListener::bind("127.0.0.1:0").await {
            Ok(l) => l,
            Err(e) => return tracing::warn!("mod_link: could not open a port: {e}"),
        };
        let port = match listener.local_addr() {
            Ok(a) => a.port(),
            Err(e) => return tracing::warn!("mod_link: could not read the address: {e}"),
        };
        let key = handshake::new_key();
        if let Err(e) = handshake::write(&instance_dir, port, &key).await {
            return tracing::warn!("mod_link: handshake file not written: {e}");
        }

        let accept = tokio::spawn(server::serve(ctx.clone(), self.clone(), listener));
        let mut guard = self.inner.lock();
        guard.key = key;
        guard.instance_dir = Some(instance_dir);
        guard.accept = Some(accept);
        tracing::info!(port, "mod_link: channel open");
    }

    /// The game closed: shut the listener down and drop the key.
    pub async fn stop(&self) {
        let (accept, dir) = {
            let mut guard = self.inner.lock();
            guard.key.clear();
            guard.tx = None;
            (guard.accept.take(), guard.instance_dir.take())
        };
        if let Some(accept) = accept {
            accept.abort();
        }
        if let Some(dir) = dir {
            handshake::remove(&dir).await;
        }
        self.store.close();
    }

    /// The master says a case changed. Re-read whatever is on screen.
    pub fn case_updated(&self, ctx: &Ctx, case_id: Uuid) {
        let open = self.store.is_open(case_id);
        let (ctx, link) = (ctx.clone(), self.clone());
        tokio::spawn(async move {
            // A case that isn't on screen refreshes when it's opened. The queue
            // is re-read either way — its row for this case may have changed.
            if open {
                push::refresh_case(&ctx, &link, case_id).await;
            }
            push::refresh_queue_in_place(&ctx, &link).await;
        });
    }

    pub fn send(&self, frame: ToMod) {
        if let Some(tx) = &self.inner.lock().tx {
            let _ = tx.send(frame);
        }
    }

    fn key(&self) -> String {
        self.inner.lock().key.clone()
    }

    fn attach(&self, tx: UnboundedSender<ToMod>) {
        self.inner.lock().tx = Some(tx);
    }

    fn detach(&self) {
        self.inner.lock().tx = None;
        self.store.close();
    }
}
