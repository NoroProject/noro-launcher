//! Where the launcher keeps its data on disk.

use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct LauncherDirectories {
    /// `~/.local/share/noro`, `%APPDATA%/noro`, `~/Library/Application Support/noro`.
    pub root: PathBuf,
}

impl LauncherDirectories {
    pub fn new() -> Self {
        let root = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(schema::launcher_dir_name());
        Self { root }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
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

    pub fn instances(&self) -> PathBuf {
        self.root.join("instances")
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

    pub fn ensure(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.instances())?;
        std::fs::create_dir_all(self.updates())?;
        Ok(())
    }
}

impl Default for LauncherDirectories {
    fn default() -> Self {
        Self::new()
    }
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
