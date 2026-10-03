//! File hashing.

use anyhow::Result;
use sha1::{Digest, Sha1};
use std::io::Read;
use std::path::Path;

/// Hashed on the blocking pool with plain `std::fs` and a large buffer. Through
/// `tokio::fs` every 64 KB read was its own round trip to that pool, and the
/// hashing itself ran on a runtime worker.
pub async fn sha1_file(path: &Path) -> Result<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut file = std::fs::File::open(&path)?;
        let mut hasher = Sha1::new();
        let mut buf = vec![0u8; 1024 * 1024];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(hex::encode(hasher.finalize()))
    })
    .await?
}

/// For launcher update binaries — those are published with a sha256, not a sha1.
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::Sha256;
    hex::encode(Sha256::digest(data))
}
