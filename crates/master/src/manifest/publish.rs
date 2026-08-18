//! Служебная сборка манифеста: подпись метаданных сборки и сводка для админки.

use super::build_manifest;
use crate::db::models::BuildRow;
use crate::state::AppState;
use anyhow::{Context, Result};
use schema::BuildManifest;

/// Сериализовать манифест и проверить, что он валиден (для отладки публикации).
pub fn manifest_summary(m: &BuildManifest) -> String {
    let total: u64 = m.verified_files.iter().map(|f| f.size).sum();
    format!(
        "build {} v{} ({} {}): {} файлов, {:.1} МБ",
        m.build_id,
        m.version,
        m.modloader.as_str(),
        m.mc_version,
        m.verified_files.len(),
        total as f64 / 1_048_576.0
    )
}

/// Гарантировать, что у сборки заполнен manifest_signature; пересобрать при нужде.
///
/// Здесь `viewer` = `None`: подписывается полный набор, потому что это
/// метаданные сборки, а не то, что уедет игроку.
pub async fn ensure_signed(state: &AppState, build: &BuildRow) -> Result<BuildManifest> {
    let manifest = build_manifest(state, build, None, "").await?;
    crate::db::update_build_manifest_meta(
        &state.db,
        build.id,
        &manifest.main_class,
        &serde_json::to_value(&manifest.jvm_args)?,
        &serde_json::to_value(&manifest.game_args)?,
        &manifest.assets_index_name,
        &manifest.signature,
    )
    .await
    .context("сохранение метаданных манифеста")?;
    Ok(manifest)
}
