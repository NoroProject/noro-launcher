// Over 150 lines: deciding whether an announced address can be trusted, then
// recording the move in both files that remember the address.
//! Following the master to a new address.
//!
//! An installed bootstrapper carries the address it was stamped with and never
//! updates, so after the master changes domain it keeps handing core the old
//! name on every start. The master announces its canonical address in a
//! header on every response; core compares it with its own and moves:
//! `config.json` takes the new address, the old origin goes on a list that
//! [`LauncherConfig::adopt_stamped_master`] refuses to go back to, and
//! `bootstrap.json` is rewritten so a bootstrapper that knows about moves
//! starts from the new address too.
//!
//! The running session is not reconnected. The old name keeps working for as
//! long as installed bootstrappers need it, so nothing breaks by finishing the
//! session there; requests made from here on already use the new address, and
//! the socket and the window pick it up on the next start.

use crate::config::LauncherConfig;
use reqwest::Url;
use std::path::Path;

/// Set by the master on every response.
pub const HEADER: &str = "x-noro-master";

/// `scheme://host[:port]` of an http(s) address, lowercased, default port
/// dropped. Comparing origins rather than strings keeps a trailing slash or
/// a capital letter from looking like a move.
pub fn origin(url: &str) -> Option<String> {
    let url = Url::parse(url.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return None;
    }
    Some(url.origin().ascii_serialization())
}

/// The address `header` asks core to move to, or `None` when there is no move
/// or the announcement can't be trusted.
///
/// Trusted means the answer came over TLS from the address core is using —
/// not after a redirect to somewhere else — and names a bare https origin. A
/// debug build also takes the answer over plain http from a local master, so
/// this can be tried against one; the target is https all the same.
pub fn announced(current: &str, response_url: &Url, header: Option<&str>) -> Option<String> {
    let current = origin(current)?;
    if response_url.origin().ascii_serialization() != current || !trusted_channel(response_url) {
        return None;
    }
    let target = Url::parse(header?.trim()).ok()?;
    let bare = target.scheme() == "https"
        && target.host_str().is_some()
        && target.username().is_empty()
        && target.password().is_none()
        && target.path() == "/"
        && target.query().is_none()
        && target.fragment().is_none();
    let target = target.origin().ascii_serialization();
    (bare && target != current).then_some(target)
}

fn trusted_channel(url: &Url) -> bool {
    match url.scheme() {
        "https" => true,
        "http" => {
            cfg!(debug_assertions)
                && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
        }
        _ => false,
    }
}

impl LauncherConfig {
    /// Whether `url` is an address the master has moved away from.
    pub fn moved_away_from(&self, url: &str) -> bool {
        origin(url).is_some_and(|o| self.master_moved_from.contains(&o))
    }

    /// Switch to `target`, remembering where the master was. Returns the
    /// address left behind.
    pub fn move_master(&mut self, target: &str) -> String {
        let old = std::mem::replace(&mut self.master_url, target.to_string());
        if let Some(old) = origin(&old) {
            if !self.master_moved_from.contains(&old) {
                self.master_moved_from.push(old);
            }
        }
        // Moving back to an address left earlier: it is current again, and
        // refusing it as stale would undo this move on the next start.
        if let Some(target) = origin(target) {
            self.master_moved_from.retain(|o| *o != target);
        }
        old
    }
}

/// Put the move into `bootstrap.json`, keeping every other field — the
/// bootstrapper writes its signing key there. Returns whether the file
/// changed; one that already says this is left alone.
pub fn write_bootstrap(
    path: &Path,
    master_url: &str,
    moved_from: &[String],
) -> std::io::Result<bool> {
    let mut doc = std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|v| match v {
            serde_json::Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default();
    let before = doc.clone();
    doc.insert("master_url".into(), master_url.into());
    doc.insert("moved_from".into(), moved_from.into());
    if doc == before {
        return Ok(false);
    }
    let json = serde_json::to_string_pretty(&doc).map_err(std::io::Error::other)?;
    crate::fsutil::write_atomic_sync(path, json.as_bytes())?;
    Ok(true)
}

/// Look at a successful answer from the master and follow it if it announces
/// a new address.
pub fn follow(ctx: &crate::backend::Ctx, res: &reqwest::Response) {
    if !res.status().is_success() {
        return;
    }
    let header = res.headers().get(HEADER).and_then(|v| v.to_str().ok());
    let current = ctx.config.get().master_url;
    let Some(target) = announced(&current, res.url(), header) else {
        return;
    };
    let mut left = String::new();
    let mut moved_from = Vec::new();
    ctx.config.update(|c| {
        left = c.move_master(&target);
        moved_from = c.master_moved_from.clone();
    });
    tracing::warn!("the master has moved: {left} -> {target}; using the new address from now on");
    sync_bootstrap(&ctx.dirs.bootstrap_file(), &target, &moved_from);
}

/// Called on start too: a bootstrapper from before moves rewrites
/// `bootstrap.json` with its stamped address on every launch.
pub fn sync_bootstrap(path: &Path, master_url: &str, moved_from: &[String]) {
    if moved_from.is_empty() {
        return;
    }
    match write_bootstrap(path, master_url, moved_from) {
        Ok(true) => tracing::info!("bootstrap.json now points at {master_url}"),
        Ok(false) => {}
        Err(e) => tracing::warn!("could not record the move in bootstrap.json: {e}"),
    }
}

#[cfg(test)]
#[path = "master_move_tests.rs"]
mod tests;
