//! Версии лаунчера и самообновление.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LauncherVersion {
    pub id: Uuid,
    pub version: String,
    /// "linux-x86_64", "macos-aarch64", "windows-x86_64".
    pub platform: String,
    pub url: String,
    pub sha256: String,
    /// ed25519-подпись бинарника, base64.
    pub signature: String,
    pub is_current: bool,
}

/// Текущая платформа в нотации мастера.
pub fn current_platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        ("windows", "x86_64") => "windows-x86_64",
        (os, arch) => {
            // Неизвестная платформа — отдадим строку как есть, мастер просто не найдёт версию.
            Box::leak(format!("{os}-{arch}").into_boxed_str())
        }
    }
}
