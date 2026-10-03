//! Writing files so that a crash, a power cut or a full disk leaves either the
//! old contents or the new ones, never a truncated mix.
//!
//! A plain `fs::write` truncates first and writes second. Interrupted in
//! between, the next start reads an empty or half-written JSON, and every
//! loader here falls back to defaults: the player's settings, pinned builds and
//! merge bases are gone without a word.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Two writes of the same file from this process must not share a temp name.
static NEXT: AtomicU64 = AtomicU64::new(0);

fn temp_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(format!(
        ".{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    path.with_file_name(name)
}

/// Write to a sibling temp file, flush it to disk, then rename it over `path`.
/// The rename is atomic on the same filesystem, which a sibling always is.
pub fn write_atomic_sync(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = temp_path(path);
    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// [`write_atomic_sync`] on the blocking pool, so the fsync doesn't hold up a
/// runtime worker.
pub async fn write_atomic(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    let path = path.as_ref().to_path_buf();
    let bytes = bytes.as_ref().to_vec();
    tokio::task::spawn_blocking(move || write_atomic_sync(&path, &bytes))
        .await
        .map_err(std::io::Error::other)?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        std::env::temp_dir().join(format!("noro-fsutil-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn replaces_the_file_and_leaves_no_temp_behind() {
        let dir = scratch();
        let path = dir.join("nested/config.json");

        write_atomic_sync(&path, b"first").unwrap();
        write_atomic_sync(&path, b"second").unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"second");
        let names: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from("config.json")]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn async_variant_writes_too() {
        let dir = scratch();
        let path = dir.join("a.txt");
        write_atomic(&path, "hello").await.unwrap();
        assert_eq!(tokio::fs::read_to_string(&path).await.unwrap(), "hello");
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
