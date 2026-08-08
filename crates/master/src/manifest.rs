//! Сборка и подпись BuildManifest из данных БД.
//!
//! `build_files` хранит ПОЛНЫЙ verified-набор сборки: моды/конфиги + mc.jar +
//! библиотеки + natives + assets + java. Колонка `kind` различает их.
//! `mojang_artifacts` — лишь кеш скачивания, чтобы не качать одно и то же дважды.

use crate::db::models::BuildRow;
use crate::state::AppState;
use anyhow::{Context, Result};
use schema::{
    ArtifactKind, BuildManifest, FileEntry, FileSide, Modloader, OptionalMod,
    RecommendedClientSettings,
};
use std::collections::BTreeMap;
use std::str::FromStr;

/// Преобразовать строковый kind из БД в ArtifactKind.
pub fn kind_from_str(s: &str) -> ArtifactKind {
    match s {
        "client_jar" => ArtifactKind::ClientJar,
        "library" => ArtifactKind::Library,
        "runtime" => ArtifactKind::Runtime,
        "native" => ArtifactKind::Native,
        "asset" => ArtifactKind::Asset,
        "asset_index" => ArtifactKind::AssetIndex,
        "java" => ArtifactKind::Java,
        "mod" => ArtifactKind::Mod,
        "config" => ArtifactKind::Config,
        _ => ArtifactKind::Other,
    }
}

pub fn kind_to_str(k: ArtifactKind) -> &'static str {
    match k {
        ArtifactKind::ClientJar => "client_jar",
        ArtifactKind::Library => "library",
        ArtifactKind::Runtime => "runtime",
        ArtifactKind::Native => "native",
        ArtifactKind::Asset => "asset",
        ArtifactKind::AssetIndex => "asset_index",
        ArtifactKind::Java => "java",
        ArtifactKind::Mod => "mod",
        ArtifactKind::Config => "config",
        ArtifactKind::Other => "other",
    }
}

fn side_from_str(s: &str) -> FileSide {
    match s {
        "client" => FileSide::Client,
        "server" => FileSide::Server,
        _ => FileSide::Both,
    }
}

/// Собрать подписанный манифест для сборки.
pub async fn build_manifest(state: &AppState, build: &BuildRow) -> Result<BuildManifest> {
    let base_build = crate::db::get_base_build(
        &state.db,
        &build.mc_version,
        &build.modloader,
        build.modloader_version.as_deref(),
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("Base build не найден (запустите bootstrap)"))?;

    let base_files = crate::db::base_build_files(&state.db, base_build.id).await?;
    let build_files = crate::db::build_files(&state.db, build.id).await?;

    let mut verified_files = Vec::with_capacity(base_files.len() + build_files.len());
    let mut artifact_kinds = BTreeMap::new();

    // Функция-хелпер для добавления файла в итоговый манифест.
    let mut add_file = |path: String,
                        sha1: String,
                        size: i64,
                        side_str: String,
                        kind_str: String,
                        platform: Option<String>| {
        let kind = kind_from_str(&kind_str);
        artifact_kinds.insert(path.clone(), kind);
        // java-бинарники нужно делать исполняемыми на unix.
        let executable = kind == ArtifactKind::Java
            && (path.ends_with("/java") || path.ends_with("/javaw") || path.ends_with("/java.exe"));
        verified_files.push(FileEntry {
            path,
            sha1: sha1.clone(),
            size: size as u64,
            url: state.config.file_url(&sha1),
            side: side_from_str(&side_str),
            executable,
            platform,
        });
    };

    for f in base_files {
        add_file(f.path, f.sha1, f.size, f.side, f.kind, f.platform);
    }
    // Файлы самой сборки — моды и конфиги, они одинаковы для всех платформ.
    for f in build_files {
        add_file(f.path, f.sha1, f.size, f.side, f.kind, None);
    }

    let optional_mods: Vec<OptionalMod> =
        serde_json::from_value(build.optional_mods.clone()).unwrap_or_default();
    let unmanaged_paths: Vec<String> =
        serde_json::from_value(build.unmanaged_paths.clone()).unwrap_or_default();
    let user_managed_paths: Vec<String> =
        serde_json::from_value(build.user_managed_paths.clone()).unwrap_or_default();

    // Объединяем аргументы: из base_build (ванилла/лоадер) + из build (если есть кастомные)
    // Сейчас берем напрямую из base_build.
    let jvm_args: Vec<schema::ManifestArg> =
        serde_json::from_value(base_build.jvm_args.clone()).unwrap_or_default();
    let game_args: Vec<schema::ManifestArg> =
        serde_json::from_value(base_build.game_args.clone()).unwrap_or_default();

    let mut manifest = BuildManifest {
        build_id: build.id,
        server_id: build.server_id,
        version: build.version.clone(),
        mc_version: build.mc_version.clone(),
        modloader: Modloader::from_str(&build.modloader).unwrap_or(Modloader::Vanilla),
        modloader_version: build.modloader_version.clone(),
        main_class: base_build.main_class.clone(),
        jvm_args,
        game_args,
        assets_index_name: base_build.assets_index_name.clone(),
        verified_files,
        artifact_kinds,
        unmanaged_paths,
        user_managed_paths,
        optional_mods,
        recommended_client_settings: RecommendedClientSettings {
            memory_min_mb: build.recommended_memory_min_mb.max(512) as u32,
            memory_max_mb: build
                .recommended_memory_max_mb
                .max(build.recommended_memory_min_mb)
                .max(512) as u32,
            jvm_flags: build.recommended_jvm_flags.clone(),
            show_console_on_launch: build.recommended_show_console_on_launch,
        },
        signature: Vec::new(),
    };

    state.signer.sign_manifest(&mut manifest);
    Ok(manifest)
}

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
pub async fn ensure_signed(state: &AppState, build: &BuildRow) -> Result<BuildManifest> {
    let manifest = build_manifest(state, build).await?;
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
