//! The handshake file in the instance directory: a port and a one-shot key.
//!
//! It lives there rather than in the launcher's config because `gameDir` is the
//! only path the mod knows. The key isn't optional: CORS doesn't apply to
//! WebSockets, so any page can open `ws://127.0.0.1:port`. What it can't do is
//! read a file out of the game directory.

use anyhow::Result;
use mod_link::{Handshake, HANDSHAKE_FILE, PROTOCOL};
use std::path::{Path, PathBuf};

/// Two v4 UUIDs back to back: 256 bits from the system generator, without
/// pulling `rand` in just for this.
pub fn new_key() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

pub fn path(instance_dir: &Path) -> PathBuf {
    instance_dir.join(HANDSHAKE_FILE)
}

pub async fn write(instance_dir: &Path, port: u16, key: &str) -> Result<()> {
    let body = serde_json::to_vec_pretty(&Handshake {
        port,
        key: key.to_string(),
        protocol: PROTOCOL,
    })?;
    tokio::fs::write(path(instance_dir), body).await?;
    Ok(())
}

/// Once the game is closed the key is dead; leaving the file behind advertises
/// access that no longer exists.
pub async fn remove(instance_dir: &Path) {
    let _ = tokio::fs::remove_file(path(instance_dir)).await;
}
