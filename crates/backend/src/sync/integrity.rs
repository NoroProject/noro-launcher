//! File hashing.

use anyhow::Result;
use sha1::{Digest, Sha1};
use std::path::Path;
use tokio::io::AsyncReadExt;

pub async fn sha1_file(path: &Path) -> Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// For launcher update binaries — those are published with a sha256, not a sha1.
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::Sha256;
    hex::encode(Sha256::digest(data))
}
