//! Подготовка артефактов сборки: minecraft.jar, библиотеки, natives, assets, java
//! и модлоадер. Всё кладётся в FileStore (с дедупликацией по SHA1) и регистрируется
//! в `build_files` как полный verified-набор сборки.
//!
//! Java и natives готовятся под платформу мастера (`current_platform()`). Для
//! single-platform-деплоя (Linux-сервер → Linux/локальные клиенты той же платформы)
//! этого достаточно; кросс-платформенность — точка расширения.

pub mod assets;
pub mod fabric;
pub mod forge;
pub mod java;
pub mod maven;
pub mod minecraft;
pub mod platform;
mod processor_java;

use crate::state::AppState;
use anyhow::{Context, Result};
use schema::{ArtifactKind, Modloader};
use std::str::FromStr;

/// Контекст одной операции bootstrap для конкретной сборки.
pub struct BootstrapCtx<'a> {
    pub state: &'a AppState,
    pub base_build_id: uuid::Uuid,
    pub mc_version: String,
    pub modloader: Modloader,
    pub modloader_version: Option<String>,
    /// Платформа клиента (для java/natives).
    pub platform: platform::Platform,
    /// Главный класс (определяется модлоадером).
    pub main_class: String,
    pub jvm_args: Vec<schema::ManifestArg>,
    pub game_args: Vec<schema::ManifestArg>,
    pub assets_index_name: String,
    /// Не доверять содержимому стора: качать всё заново и перезаписывать.
    /// Ставится только пересбором «с нуля» из админки.
    pub refetch: bool,
    /// Опциональная функция логирования прогресса (для админки/CLI).
    pub log: Box<dyn Fn(&str) + Send + Sync + 'a>,
}

impl<'a> BootstrapCtx<'a> {
    pub fn logf(&self, msg: impl AsRef<str>) {
        (self.log)(msg.as_ref());
    }

    /// Скачать (если нужно) и зарегистрировать один артефакт.
    /// `path` — относительный путь в игровой директории.
    pub async fn register(
        &self,
        path: &str,
        kind: ArtifactKind,
        side: &str,
        url: &str,
        expected_sha1: Option<&str>,
        _artifact_type: &str,
    ) -> Result<()> {
        let files = &self.state.files;
        let stored = if self.refetch {
            files
                .put_url_overwriting(self.state.http(), url, expected_sha1)
                .await
        } else {
            files.put_url(self.state.http(), url, expected_sha1).await
        }
        .with_context(|| format!("скачивание {path}"))?;

        crate::db::upsert_base_build_file(
            &self.state.db,
            self.base_build_id,
            path,
            &stored.sha1,
            stored.size as i64,
            side,
            crate::manifest::kind_to_str(kind),
            platform_tag(kind, self.platform),
        )
        .await?;
        Ok(())
    }

    /// Зарегистрировать уже сохранённые байты (для пропатченных forge jar и т.п.).
    pub async fn register_bytes(
        &self,
        path: &str,
        kind: ArtifactKind,
        side: &str,
        data: &[u8],
        _artifact_type: &str,
    ) -> Result<String> {
        let files = &self.state.files;
        let stored = if self.refetch {
            files.put_bytes_overwriting(data).await?
        } else {
            files.put_bytes(data).await?
        };
        crate::db::upsert_base_build_file(
            &self.state.db,
            self.base_build_id,
            path,
            &stored.sha1,
            stored.size as i64,
            side,
            crate::manifest::kind_to_str(kind),
            platform_tag(kind, self.platform),
        )
        .await?;
        Ok(stored.sha1)
    }
}

/// Java-рантайм и natives — разные бинарники под каждую ОС, остальное одинаково
/// всюду. Помечаем только их: клиент скачает свой набор, а не пять чужих.
fn platform_tag(kind: ArtifactKind, platform: platform::Platform) -> Option<&'static str> {
    matches!(kind, ArtifactKind::Java | ArtifactKind::Native).then(|| platform.tag())
}

/// Гарантировать наличие готового base_build. Если его нет — скачивает и собирает все
/// необходимые файлы майна/модлоадера в `base_build_files`.
pub async fn ensure_base_build<F>(
    state: &AppState,
    mc_version: &str,
    modloader: &str,
    modloader_version: Option<&str>,
    log: F,
) -> Result<crate::db::models::BaseBuildRow>
where
    F: Fn(&str) + Send + Sync,
{
    // 1. Проверяем, есть ли уже готовый base build.
    if let Some(base) =
        crate::db::get_base_build(&state.db, mc_version, modloader, modloader_version).await?
    {
        // Сборки, сделанные до мультиплатформенности, несут java только под ОС
        // мастера. Переиспользовать их нельзя: на других системах JVM не
        // запустится, а сами по себе они не починятся — bootstrap для готовой
        // сборки не запускается.
        if !crate::db::has_legacy_java(&state.db, base.id).await? {
            log("Base build уже существует, пропуск bootstrap");
            return Ok(base);
        }
        let removed = crate::db::delete_base_build_files_kind(&state.db, base.id, "java").await?;
        log(&format!(
            "java разложена под одну платформу — перекладываем ({removed} файлов удалено)"
        ));
    }

    bootstrap(state, mc_version, modloader, modloader_version, false, log).await
}

/// Собрать base_build заново, не переиспользуя ничего из прошлого раза.
///
/// Нужно, когда испортилось то, что `ensure_base_build` считает готовым и
/// поэтому не трогает: свёрнутые под одну ОС аргументы (до 1.3.1 мастер
/// вычислял `rules` у себя), битый блоб в сторе, оборвавшийся на середине
/// прошлый bootstrap. Запись со всеми её файлами удаляется, артефакты качаются
/// заново мимо кэша стора.
///
/// Файлы самой сборки — моды, конфиги, всё загруженное и импортированное — не
/// трогает: воссоздать их bootstrap не может, они не выводятся из манифестов.
pub async fn rebuild_base_build<F>(
    state: &AppState,
    mc_version: &str,
    modloader: &str,
    modloader_version: Option<&str>,
    log: F,
) -> Result<crate::db::models::BaseBuildRow>
where
    F: Fn(&str) + Send + Sync,
{
    let removed =
        crate::db::delete_base_build(&state.db, mc_version, modloader, modloader_version).await?;
    log(&format!(
        "снесён прошлый base build ({removed} записей), качаем заново мимо кэша"
    ));

    bootstrap(state, mc_version, modloader, modloader_version, true, log).await
}

async fn bootstrap<F>(
    state: &AppState,
    mc_version: &str,
    modloader: &str,
    modloader_version: Option<&str>,
    refetch: bool,
    log: F,
) -> Result<crate::db::models::BaseBuildRow>
where
    F: Fn(&str) + Send + Sync,
{
    // Временная запись — чтобы получить UUID для файлов.
    let base_build_id = crate::db::upsert_base_build(
        &state.db,
        mc_version,
        modloader,
        modloader_version,
        "",
        &serde_json::Value::Array(Vec::new()),
        &serde_json::Value::Array(Vec::new()),
        "",
    )
    .await?;

    let parsed_loader = Modloader::from_str(modloader).unwrap_or(Modloader::Vanilla);
    let mut ctx = BootstrapCtx {
        state,
        base_build_id,
        mc_version: mc_version.to_string(),
        modloader: parsed_loader,
        modloader_version: modloader_version.map(String::from),
        platform: platform::Platform::host(),
        main_class: String::new(),
        jvm_args: Vec::new(),
        game_args: Vec::new(),
        assets_index_name: mc_version.to_string(),
        refetch,
        log: Box::new(log),
    };

    ctx.logf(format!(
        "bootstrap глобальной сборки ({} {} {})",
        parsed_loader.as_str(),
        mc_version,
        modloader_version.unwrap_or("")
    ));

    // 1-4, 6-8: ванильные артефакты (jar, libs, natives, assets, java).
    let version_json = minecraft::bootstrap_vanilla(&mut ctx).await?;

    // 5: модлоадер.
    match parsed_loader {
        Modloader::Vanilla => {}
        Modloader::Fabric | Modloader::Quilt => {
            fabric::bootstrap_fabric(&mut ctx, parsed_loader).await?;
        }
        Modloader::Forge | Modloader::NeoForge => {
            forge::bootstrap_forge(&mut ctx, parsed_loader, &version_json).await?;
        }
    }

    // Сохраняем итоговые аргументы и mainClass.
    crate::db::upsert_base_build(
        &state.db,
        mc_version,
        modloader,
        modloader_version,
        &ctx.main_class,
        &serde_json::to_value(&ctx.jvm_args)?,
        &serde_json::to_value(&ctx.game_args)?,
        &ctx.assets_index_name,
    )
    .await?;

    ctx.logf("Глобальный bootstrap завершён");

    // Возвращаем обновленный base_build.
    let base = crate::db::get_base_build(&state.db, mc_version, modloader, modloader_version)
        .await?
        .unwrap();
    Ok(base)
}
