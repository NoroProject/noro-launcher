//! Сборка и подпись BuildManifest из данных БД.
//!
//! `build_files` хранит ПОЛНЫЙ verified-набор сборки: моды/конфиги + mc.jar +
//! библиотеки + natives + assets + java. Колонка `kind` различает их.
//! `mojang_artifacts` — лишь кеш скачивания, чтобы не качать одно и то же дважды.

pub mod access;
mod kinds;
mod publish;

use crate::db::models::BuildRow;
use crate::state::AppState;
use anyhow::{Context, Result};
use kinds::side_from_str;
pub use kinds::{kind_from_str, kind_to_str};
pub use publish::{ensure_signed, manifest_summary};
use schema::{
    ArtifactKind, BuildManifest, FileEntry, Modloader, OptionalMod, RecommendedClientSettings,
    UserProfile,
};

use std::collections::BTreeMap;
use std::str::FromStr;

/// Собрать подписанный манифест для сборки.
///
/// `viewer` — игрок, которому манифест уедет. `None` означает служебную сборку
/// (публикация, пересборка): фильтрация по правам не применяется, потому что
/// адресата нет. Всё, что уходит игроку, обязано передавать профиль.
pub async fn build_manifest(
    state: &AppState,
    build: &BuildRow,
    viewer: Option<&UserProfile>,
    platform: &str,
) -> Result<BuildManifest> {
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

    // Всё ниже уезжает в подписанный манифест. Пустой список вместо непрочитанного
    // JSON — это подпись под выдуманным содержимым: игрок получил бы сборку без
    // модов или без jvm-аргументов и с валидной подписью на ней.
    let optional_mods: Vec<OptionalMod> = serde_json::from_value(build.optional_mods.clone())
        .context("optional_mods сборки не разобрать")?;
    let unmanaged_paths: Vec<String> = serde_json::from_value(build.unmanaged_paths.clone())
        .context("unmanaged_paths сборки не разобрать")?;
    let user_managed_paths: Vec<String> = serde_json::from_value(build.user_managed_paths.clone())
        .context("user_managed_paths сборки не разобрать")?;
    // Правила либо заданы явно, либо выводятся из двух старых списков. Второе —
    // не «на всякий случай»: у всех существующих сборок правил ещё нет.
    let path_rules: Vec<schema::PathRule> = match &build.path_rules {
        Some(v) => serde_json::from_value(v.clone()).context("path_rules сборки не разобрать")?,
        None => schema::from_legacy(&unmanaged_paths, &user_managed_paths),
    };

    // Объединяем аргументы: из base_build (ванилла/лоадер) + из build (если есть кастомные)
    // Сейчас берем напрямую из base_build.
    let jvm_args: Vec<schema::ManifestArg> = serde_json::from_value(base_build.jvm_args.clone())
        .context("jvm_args базовой сборки не разобрать")?;
    let game_args: Vec<schema::ManifestArg> = serde_json::from_value(base_build.game_args.clone())
        .context("game_args базовой сборки не разобрать")?;

    // Внутрь подписи: список, который можно подменить на клиенте, ничего не
    // запрещает.
    let blocked_files = crate::db::blocked_files_for(&state.db, build.server_id).await?;

    let mut manifest = BuildManifest {
        build_id: build.id,
        server_id: build.server_id,
        version: build.version.clone(),
        mc_version: build.mc_version.clone(),
        // Vanilla по умолчанию означала бы «запусти без загрузчика» — игра
        // стартует и падает без единого мода вместо внятного отказа.
        modloader: Modloader::from_str(&build.modloader)
            .map_err(|_| anyhow::anyhow!("неизвестный загрузчик в сборке: {}", build.modloader))?,
        modloader_version: build.modloader_version.clone(),
        main_class: base_build.main_class.clone(),
        jvm_args,
        game_args,
        assets_index_name: base_build.assets_index_name.clone(),
        verified_files,
        artifact_kinds,
        unmanaged_paths,
        user_managed_paths,
        path_rules,
        blocked_files,
        optional_mods,
        allow_optional_mod_suggestions: build.allow_optional_mod_suggestions,
        recommended_client_settings: RecommendedClientSettings {
            memory_min_mb: build.recommended_memory_min_mb.max(512) as u32,
            memory_max_mb: build
                .recommended_memory_max_mb
                .max(build.recommended_memory_min_mb)
                .max(512) as u32,
            jvm_flags: build.recommended_jvm_flags.clone(),
            show_console_on_launch: build.recommended_show_console_on_launch,
            fullscreen: false,
        },
        signature: Vec::new(),
    };

    // До подписи: подпись должна покрывать ровно тот набор, который уедет.
    if let Some(v) = viewer {
        access::filter_for_viewer(&mut manifest, v, access::os_of(platform));
    }
    state.signer.sign_manifest(&mut manifest);
    Ok(manifest)
}
