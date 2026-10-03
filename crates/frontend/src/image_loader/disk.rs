//! Pictures kept on disk by URL. Every start used to download every icon and
//! background again, and offline the window had none at all.

use std::path::PathBuf;
use std::time::Duration;

/// Younger than this is used without asking the network.
pub const FRESH: Duration = Duration::from_secs(24 * 3600);
/// Not refreshed for this long: nothing shows it any more.
const KEEP: Duration = Duration::from_secs(30 * 24 * 3600);

fn dir() -> Option<PathBuf> {
    Some(
        dirs::data_dir()?
            .join(schema::launcher_dir_name())
            .join("cache")
            .join("images"),
    )
}

pub fn path(url: &str) -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut h);
    Some(dir()?.join(format!("{:016x}", h.finish())))
}

fn type_path(path: &std::path::Path) -> PathBuf {
    path.with_extension("type")
}

/// The bytes, their content type if one was recorded, and their age.
pub fn read(path: &std::path::Path) -> Option<(Vec<u8>, Option<String>, Duration)> {
    let age = std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .elapsed()
        .unwrap_or_default();
    let bytes = std::fs::read(path).ok()?;
    let content_type = std::fs::read_to_string(type_path(path)).ok();
    Some((bytes, content_type, age))
}

pub fn write(path: &std::path::Path, bytes: &[u8], content_type: Option<&str>) {
    let Some(parent) = path.parent() else {
        return;
    };
    let _ = std::fs::create_dir_all(parent);
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, bytes).is_ok() && std::fs::rename(&tmp, path).is_ok() {
        match content_type {
            Some(t) => {
                let _ = std::fs::write(type_path(path), t);
            }
            None => {
                let _ = std::fs::remove_file(type_path(path));
            }
        }
    }
}

/// Once per run, in the background.
pub fn prune() {
    let Some(dir) = dir() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > KEEP);
        if stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}
