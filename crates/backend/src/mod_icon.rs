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
