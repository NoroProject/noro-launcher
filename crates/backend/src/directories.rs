//! Расположение данных лаунчера на диске.

use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct LauncherDirectories {
    /// Корень данных (~/.local/share/noro, %APPDATA%/noro, ~/Library/Application Support/noro).
    pub root: PathBuf,
}

impl LauncherDirectories {
    pub fn new() -> Self {
        let root = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(schema::launcher_dir_name());
        Self { root }
    }

    /// Корень данных лаунчера.
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Файл конфигурации.
    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.json")
    }

    /// Файл выбранных опциональных модов (по серверам).
    pub fn optional_mods_file(&self) -> PathBuf {
        self.root.join("optional_mods.json")
    }

    /// Корень игровых инстансов.
    pub fn instances(&self) -> PathBuf {
        self.root.join("instances")
    }

    /// Игровая директория конкретного сервера.
    pub fn instance(&self, server_id: &uuid::Uuid) -> PathBuf {
        self.instances().join(server_id.to_string())
    }

    /// Директория для распакованных natives при запуске.
    pub fn natives(&self, server_id: &uuid::Uuid) -> PathBuf {
        self.instance(server_id).join(".natives")
    }

    /// Путь к authlib-injector jar.
    pub fn authlib_injector(&self) -> PathBuf {
        self.root.join("authlib-injector.jar")
    }

    /// Каталог для новой версии лаунчера при обновлении.
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

/// Безопасно соединить базовый путь с относительным, не давая выйти за пределы (../).
pub fn safe_join(base: &Path, rel: &str) -> Option<PathBuf> {
    let mut result = base.to_path_buf();
    for comp in Path::new(rel).components() {
        match comp {
            std::path::Component::Normal(c) => result.push(c),
            std::path::Component::CurDir => {}
            // Запрещаем ParentDir/RootDir/Prefix — защита от path traversal.
            _ => return None,
        }
    }
    Some(result)
}
