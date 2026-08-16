//! Копии исходных версий конфигов.
//!
//! Слияние по ключам, в отличие от файлового three-way, требует не хеш, а сам
//! текст того, что установил сервер в прошлый раз.

use super::is_mergeable;
use std::path::Path;

/// Где лежит копия исходного файла для три-стороннего слияния.
pub fn base_copy_path(instance_dir: &Path, rel: &str) -> std::path::PathBuf {
    instance_dir.join(".noro/base").join(rel)
}

/// Отложить то, что установил сервер, — это и станет базой в следующий раз.
///
/// Копия делается только для путей, где слияние по ключам вообще применимо:
/// хранить копии всех конфигов ради формата, который мы не умеем сливать, —
/// цена без выигрыша.
pub async fn remember_base(instance_dir: &Path, rel: &str) {
    if !is_mergeable(rel) {
        return;
    }
    let dst = base_copy_path(instance_dir, rel);
    if let Some(parent) = dst.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let _ = tokio::fs::copy(instance_dir.join(rel), dst).await;
}
