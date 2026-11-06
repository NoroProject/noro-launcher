// Over 150 lines: the two roots and every path under them, listed in one
// place so nothing builds a launcher path on its own.
//! Where the launcher keeps its data on disk.
//!
//! Two roots. The *service* root never moves: installed bootstrappers never
//! update, and they look for core, its signature, `version`, `bootstrap.json`
//! and `config.json` there by a fixed path. The *data* root holds what is
//! heavy — the instances with their Java, libraries, assets, mods, worlds and
//! screenshots, and the shared store — and may be moved to another disk (see
//! [`crate::data_location`]). Until the player moves it, it is the service root.

use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// What lives under the data root. Everything else stays in the service root.
pub const DATA_ENTRIES: [&str; 2] = ["instances", "store"];

/// Written into a data root that is not the service root. A disk that comes
/// back under the same letter or mount point but with other contents must not
/// pass for ours.
pub const DATA_MARKER: &str = ".noro-data";

#[derive(Clone)]
pub struct LauncherDirectories {
    /// `~/.local/share/noro`, `%APPDATA%/noro`, `~/Library/Application Support/noro`.
    root: PathBuf,
    /// Shared between clones: a finished move switches every task's paths at
    /// once, instead of leaving copies of the old one in background tasks.
    data: Arc<RwLock<PathBuf>>,
}

impl LauncherDirectories {
    pub fn new() -> Self {
        Self::at(service_root())
    }

    /// Both roots in one folder, the way a fresh install has them.
    pub fn at(root: PathBuf) -> Self {
        Self {
            data: Arc::new(RwLock::new(root.clone())),
            root,
        }
    }

    /// The service root: core, configs, keys, logs, caches.
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn data_root(&self) -> PathBuf {
        self.data.read().clone()
    }

    pub fn data_root_is_default(&self) -> bool {
        *self.data.read() == self.root
    }

    /// `None` goes back to the service root.
    pub fn set_data_root(&self, path: Option<&Path>) {
        *self.data.write() = path.map_or_else(|| self.root.clone(), Path::to_path_buf);
    }

    /// The default data root is always there; a moved one may sit on a disk
    /// that is not connected. Its marker, or an instances folder for a marker
    /// someone deleted, says it is ours.
    pub fn data_available(&self) -> bool {
        let data = self.data_root();
        data == self.root
            || (data.is_dir()
                && (data.join(DATA_MARKER).is_file() || data.join("instances").is_dir()))
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.json")
    }

    /// Master address for the very first run, before there is a config.
    pub fn bootstrap_file(&self) -> PathBuf {
        self.root.join("bootstrap.json")
    }

    pub fn optional_mods_file(&self) -> PathBuf {
        self.root.join("optional_mods.json")
    }

    /// A move of the data root in progress; see [`crate::data_location`].
    pub fn move_journal(&self) -> PathBuf {
        self.root.join("data-move.json")
    }

    pub fn instances(&self) -> PathBuf {
        self.data_root().join("instances")
    }

    /// Game directory for one server.
    pub fn instance(&self, server_id: &uuid::Uuid) -> PathBuf {
        self.instances().join(server_id.to_string())
    }

    /// Where natives get unpacked at launch.
    pub fn natives(&self, server_id: &uuid::Uuid) -> PathBuf {
        self.instance(server_id).join(".natives")
    }

    pub fn authlib_injector(&self) -> PathBuf {
        self.root.join("authlib-injector.jar")
    }

    /// Staging area for a downloaded launcher build.
    pub fn updates(&self) -> PathBuf {
        self.root.join("updates")
    }

    /// Shared content-addressable store across instances. Next to the
    /// instances, because it hands files to them as hard links, and those
    /// don't cross disks.
    pub fn store(&self) -> PathBuf {
        self.data_root().join("store")
    }

    /// The service folders. The data ones come from [`Self::ensure_data`], and
    /// only once the data root is known to be there: creating them on a
    /// disconnected drive's mount point would start a silent full download.
    pub fn ensure(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.updates())?;
        Ok(())
    }

    pub fn ensure_data(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.instances())?;
        std::fs::create_dir_all(self.store())?;
        Ok(())
    }
}

impl Default for LauncherDirectories {
    fn default() -> Self {
        Self::new()
    }
}

/// The fixed folder. The bootstrapper computes the same path on its own.
pub fn service_root() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(schema::launcher_dir_name())
}

/// Joins a relative path onto `base`, returning `None` for anything that could
/// climb out of it: `..`, an absolute path, or a Windows drive prefix. Paths
/// here come from manifests, so they are not trusted.
pub fn safe_join(base: &Path, rel: &str) -> Option<PathBuf> {
    let mut result = base.to_path_buf();
    for comp in Path::new(rel).components() {
        match comp {
            std::path::Component::Normal(c) => result.push(c),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    Some(result)
}

#[cfg(test)]
#[path = "directories_tests.rs"]
mod tests;
