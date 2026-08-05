//! Синхронизация файлов сервера: проверка подписи, докачка изменённого,
//! удаление лишнего (verified-set защита), обработка опциональных модов.

use super::downloader::{download_all, needs_download, DownloadTask};
use crate::directories::safe_join;
use anyhow::{bail, Result};
use bridge::SyncStage;
use schema::{ArtifactKind, BuildManifest, FileEntry, UserProfile};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Колбэк прогресса: (стадия, готово, всего, текущий файл).
pub type ProgressFn = Arc<dyn Fn(SyncStage, u64, u64, String) + Send + Sync>;

pub async fn sync_server(
    client: &reqwest::Client,
    instance_dir: &Path,
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
    progress: ProgressFn,
    cancelled: Arc<dyn Fn() -> bool + Send + Sync>,
) -> Result<()> {
    // 1. Проверка подписи манифеста.
    if !crate::signing::verify_manifest(manifest) {
        bail!("подпись манифеста недействительна — синхронизация прервана");
    }

    tokio::fs::create_dir_all(instance_dir).await?;

    // 2. Вычислить эффективный набор файлов (исключив выключенные опц. моды).
    let excluded = excluded_optional_files(manifest, enabled_optional, user);
    // Ignored раньше значил только «не удаляй»: файл из манифеста всё равно
    // скачивался и затирал правки игрока. Папка в ignored не спасала ничего,
    // что лежит внутри и пришло с сервера.
    let effective: Vec<&FileEntry> = manifest
        .verified_files
        .iter()
        .filter(|f| f.side.needed_on_client())
        .filter(|f| !excluded.contains(&f.path))
        .filter(|f| !is_protected(&f.path, &manifest.unmanaged_paths))
        // Java-рантайм и natives лежат в сборке под все платформы сразу; чужие
        // не только бесполезны, но и весят как пять лишних JRE.
        .filter(|f| f.matches_platform())
        .collect();

    // 3. Проверка файлов — что нужно скачать.
    progress(
        SyncStage::CheckingFiles,
        0,
        effective.len() as u64,
        String::new(),
    );
    let mut tasks: Vec<(ArtifactKind, DownloadTask)> = Vec::new();
    for (i, f) in effective.iter().enumerate() {
        if cancelled() {
            bail!("отменено");
        }
        let Some(dest) = safe_join(instance_dir, &f.path) else {
            continue;
        };
        let kind = manifest.kind_of(&f.path);
        let verify_hash = matches!(
            kind,
            ArtifactKind::Mod
                | ArtifactKind::Config
                | ArtifactKind::ClientJar
                | ArtifactKind::Other
        );
        // User-managed ставим один раз: дальше файл принадлежит игроку, и
        // расхождение хеша — это его правки, а не повод их затереть.
        let wanted = if is_protected(&f.path, &manifest.user_managed_paths) {
            !dest.exists()
        } else {
            needs_download(&dest, f.size, &f.sha1, verify_hash).await
        };
        if wanted {
            tasks.push((
                kind,
                DownloadTask {
                    url: f.url.clone(),
                    dest,
                    sha1: f.sha1.clone(),
                    size: f.size,
                    executable: f.executable,
                },
            ));
        }
        if i % 64 == 0 {
            progress(
                SyncStage::CheckingFiles,
                i as u64,
                effective.len() as u64,
                String::new(),
            );
        }
    }

    // 4. Скачать по категориям (для наглядных стадий).
    for (stage, kinds) in STAGE_GROUPS {
        let group: Vec<DownloadTask> = tasks
            .iter()
            .filter(|(k, _)| kinds.contains(k))
            .map(|(_, t)| t.clone())
            .collect();
        if group.is_empty() {
            continue;
        }
        let total: u64 = group.iter().map(|t| t.size).sum();
        let count = group.len();
        progress(*stage, 0, total, format!("{count} файлов"));

        let prog = progress.clone();
        let stage_copy = *stage;
        download_all(
            client,
            group,
            8,
            move |done| prog(stage_copy, done, total, String::new()),
            {
                let c = cancelled.clone();
                move || c()
            },
        )
        .await?;
    }

    // 5. Удалить лишние файлы (всё, что не в effective и не защищено).
    progress(SyncStage::Cleaning, 0, 0, String::new());
    clean_extra(instance_dir, &effective, manifest).await?;

    // Отметка о том, что именно установлено. Без неё «поставить» и «обновить»
    // не отличить от «запустить»: набор файлов на диске сам по себе не говорит,
    // какой версии сборки он соответствует.
    let _ = tokio::fs::write(version_marker(instance_dir), &manifest.version).await;

    progress(SyncStage::Done, 1, 1, String::new());
    Ok(())
}

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

/// Сопоставление стадий и категорий артефактов.
const STAGE_GROUPS: &[(SyncStage, &[ArtifactKind])] = &[
    (SyncStage::DownloadingJava, &[ArtifactKind::Java]),
    (SyncStage::DownloadingMinecraft, &[ArtifactKind::ClientJar]),
    (
        SyncStage::DownloadingLibraries,
        &[
            ArtifactKind::Library,
            ArtifactKind::Runtime,
            ArtifactKind::Native,
        ],
    ),
    (
        SyncStage::DownloadingAssets,
        &[ArtifactKind::Asset, ArtifactKind::AssetIndex],
    ),
    (
        SyncStage::DownloadingMods,
        &[ArtifactKind::Mod, ArtifactKind::Config, ArtifactKind::Other],
    ),
];

/// Пути файлов выключенных (или недоступных по правам) опциональных модов.
fn excluded_optional_files(
    manifest: &BuildManifest,
    enabled: &[String],
    user: &UserProfile,
) -> HashSet<String> {
    let mut excluded = HashSet::new();
    for m in &manifest.optional_mods {
        let user_enabled = enabled.iter().any(|n| n == &m.name);
        let allowed = user.can_use_optional(&manifest.server_id, &m.name, m.limited);
        let active = if m.limited {
            user_enabled && allowed
        } else {
            // нелимитный: по выбору пользователя, иначе по умолчанию
            if enabled.is_empty() {
                m.enabled_by_default
            } else {
                user_enabled
            }
        };
        if !active {
            for f in &m.files {
                excluded.insert(f.clone());
            }
        }
    }
    excluded
}

/// Удалить файлы, отсутствующие в effective и не попадающие под защищённые пути.
async fn clean_extra(
    instance_dir: &Path,
    effective: &[&FileEntry],
    manifest: &BuildManifest,
) -> Result<()> {
    let keep: HashSet<String> = effective.iter().map(|f| f.path.clone()).collect();
    let mut protected: Vec<String> = Vec::new();
    protected.extend(manifest.unmanaged_paths.iter().cloned());
    protected.extend(manifest.user_managed_paths.iter().cloned());
    // Служебные пути лаунчера.
    protected.push(".natives/".to_string());

    let root = instance_dir.to_path_buf();
    let to_delete = tokio::task::spawn_blocking(move || {
        let mut victims = Vec::new();
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = match entry.path().strip_prefix(&root) {
                Ok(r) => r.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };
            if keep.contains(&rel) {
                continue;
            }
            if is_protected(&rel, &protected) {
                continue;
            }
            victims.push(entry.path().to_path_buf());
        }
        victims
    })
    .await?;

    for path in to_delete {
        let _ = tokio::fs::remove_file(&path).await;
    }
    Ok(())
}

/// Защищён ли относительный путь одним из префиксов (директория с '/' или точный файл).
fn is_protected(rel: &str, protected: &[String]) -> bool {
    protected.iter().any(|p| {
        if let Some(dir) = p.strip_suffix('/') {
            rel == dir || rel.starts_with(&format!("{dir}/"))
        } else {
            rel == p
        }
    })
}

/// Найти исполняемый java-бинарник среди файлов манифеста.
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
