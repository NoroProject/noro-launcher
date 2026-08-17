//! Синхронизация файлов сервера: проверка подписи, докачка изменённого,
//! удаление лишнего (verified-set защита), обработка опциональных модов.

use super::downloader::{download_all, DownloadTask};
use anyhow::{bail, Result};
use bridge::SyncStage;
use schema::{BuildManifest, FileEntry, UserProfile};
use std::path::Path;
use std::sync::Arc;

/// Колбэк прогресса: (стадия, готово, всего, текущий файл).
pub type ProgressFn = Arc<dyn Fn(SyncStage, u64, u64, String) + Send + Sync>;
mod clean;
mod stages;
mod state;

mod tasks;

use clean::clean_extra;
pub use clean::{excluded_optional_files, is_protected};
use stages::STAGE_GROUPS;
pub use state::{build_state, find_java, version_marker};

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
        .filter(|f| schema::mode_for(&f.path, &manifest.path_rules) != schema::PathMode::Unmanaged)
        // Java-рантайм и natives лежат в сборке под все платформы сразу; чужие
        // не только бесполезны, но и весят как пять лишних JRE.
        .filter(|f| f.matches_platform())
        .collect();

    // База хешей: то, что мы установили в прошлый раз. Без неё режим `merged`
    // не отличает правки игрока от обновления сервера.
    let base = super::merge::BaseHashes::load(instance_dir).await;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();

    // 3. Проверка файлов — что нужно скачать.
    let (tasks, base) = tasks::collect(
        client,
        instance_dir,
        manifest,
        &effective,
        base,
        &stamp,
        &progress,
        &cancelled,
    )
    .await?;
    // 4. Скачать все стадии разом. Они трогают непересекающиеся файлы, поэтому
    //    ждать очереди незачем: раньше тысяча мелких ассетов простаивала, пока
    //    докачается JDK, хотя канал в это время занят одним потоком.
    let jobs = STAGE_GROUPS.iter().filter_map(|g| {
        let group: Vec<DownloadTask> = tasks
            .iter()
            .filter(|(k, _)| g.kinds.contains(k))
            .map(|(_, t)| t.clone())
            .collect();
        if group.is_empty() {
            return None;
        }
        let total: u64 = group.iter().map(|t| t.size).sum();
        // Полосу стадии нужно показать до старта, иначе она появится в UI
        // только с первым отчётом — то есть уже наполовину заполненной.
        progress(g.stage, 0, total, String::new());

        let prog = progress.clone();
        let cancelled = cancelled.clone();
        let client = client.clone();
        Some(async move {
            download_all(
                &client,
                group,
                g.concurrency,
                {
                    let prog = prog.clone();
                    move |done| prog(g.stage, done, total, String::new())
                },
                move || cancelled(),
            )
            .await?;
            // Последний порог прогресса мог не сработать — досылаем точный итог,
            // чтобы полоса не замерла на 99%.
            prog(g.stage, total, total, String::new());
            Ok::<_, anyhow::Error>(())
        })
    });
    futures::future::try_join_all(jobs).await?;

    // Копии конфигов для слияния по ключам: делаются после загрузки, когда на
    // диске уже лежит серверная версия.
    for f in &effective {
        crate::sync::keymerge::remember_base(instance_dir, &f.path).await;
    }

    // База пишется после загрузки: до неё файлов ещё нет, и запомнить их хеш
    // значило бы соврать следующему проходу.
    base.save(instance_dir).await;

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
