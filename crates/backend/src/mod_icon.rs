//! Icons of optional mods, read out of their jars.

pub use mc_mod_utils::*;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::SystemTime;

/// Jar → (mtime, icon). Opening and unzipping every optional mod's jar ran in
/// the backend's main loop on every manifest, server open and permission
/// change; a jar that didn't change has the same icon.
type IconCache = HashMap<PathBuf, (SystemTime, Option<String>)>;

static CACHE: LazyLock<Mutex<IconCache>> = LazyLock::new(Default::default);

/// What is already known about a jar, without opening it: the outer `None`
/// means it hasn't been read yet. A jar that isn't there counts as known and
/// iconless, so nobody waits on it.
pub fn known_jar_icon(path: &Path) -> Option<Option<String>> {
    let Ok(modified) = std::fs::metadata(path).and_then(|m| m.modified()) else {
        return Some(None);
    };
    match CACHE.lock().get(path) {
        Some((at, icon)) if *at == modified => Some(icon.clone()),
        _ => None,
    }
}

pub fn cached_jar_icon(path: &Path) -> Option<String> {
    let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok()?;
    if let Some((at, icon)) = CACHE.lock().get(path) {
        if *at == modified {
            return icon.clone();
        }
    }
    let icon = extract_jar_icon(path);
    CACHE
        .lock()
        .insert(path.to_path_buf(), (modified, icon.clone()));
    icon
}
