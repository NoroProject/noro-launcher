//! Что установлено в инстансе и чем его запускать.

use crate::directories::safe_join;
use schema::{ArtifactKind, BuildManifest};
use std::path::{Path, PathBuf};

/// Файл с версией установленной сборки.
pub fn version_marker(instance_dir: &Path) -> PathBuf {
    instance_dir.join(".noro-build")
}

/// Что можно сделать со сборкой: поставить, обновить или запустить.
pub fn build_state(instance_dir: &Path, manifest: &BuildManifest) -> bridge::BuildState {
    match std::fs::read_to_string(version_marker(instance_dir)) {
        Ok(installed) if installed.trim() == manifest.version => bridge::BuildState::Ready,
        Ok(_) => bridge::BuildState::Outdated,
        Err(_) => bridge::BuildState::Missing,
    }
}

/// Стадия загрузки: какие артефакты в неё входят и сколько запросов держать
/// в полёте. Единого хорошего числа нет — профили нагрузки слишком разные.
pub fn find_java(instance_dir: &Path, manifest: &BuildManifest) -> Option<PathBuf> {
    for f in &manifest.verified_files {
        // Рантаймов в манифесте теперь несколько — берём тот, что для нашей ОС.
        if !f.matches_platform() {
            continue;
        }
        if manifest.kind_of(&f.path) == ArtifactKind::Java
            && (f.path.ends_with("/bin/java") || f.path.ends_with("/bin/java.exe"))
        {
            return safe_join(instance_dir, &f.path);
        }
    }
    None
}
