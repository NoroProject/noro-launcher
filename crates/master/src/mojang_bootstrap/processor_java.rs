//! Выбор JVM для процессоров установщика Forge/NeoForge.
//!
//! Это не то же самое, что java в сборке. Рантаймы Mojang скачиваются для
//! клиентов и лежат под все пять платформ, а процессоры установщика запускает
//! сам мастер — значит нужна JVM его архитектуры.

use super::{platform::Platform, BootstrapCtx};
use anyhow::{anyhow, bail, Context, Result};
use std::path::PathBuf;

const JAVA_BIN: &str = if cfg!(windows) { "java.exe" } else { "java" };

/// Найти java для запуска процессоров.
///
/// Системная JVM идёт первой, и это принципиально: под ключом `linux` у Mojang
/// лежит только x86_64, поэтому на arm64-мастере скачанный рантайм не
/// запустится вовсе, а JVM хоста подходит по определению.
pub(super) async fn find(ctx: &BootstrapCtx<'_>) -> Result<PathBuf> {
    if let Some(system) = system().await {
        return Ok(system);
    }
    staged(ctx).await
}

/// JVM хоста. `JAVA_HOME` важнее `PATH`: в образе мастера стоит именно она.
async fn system() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("JAVA_HOME") {
        let candidate = PathBuf::from(home).join("bin").join(JAVA_BIN);
        if tokio::fs::metadata(&candidate).await.is_ok() {
            return Some(candidate);
        }
    }
    // Пробуем запустить, а не искать по PATH руками: разбор PATH пришлось бы
    // делать с оглядкой на расширения Windows, а запуск решает это сам.
    let works = tokio::process::Command::new(JAVA_BIN)
        .arg("-version")
        .output()
        .await
        .map(|out| out.status.success())
        .unwrap_or(false);
    works.then(|| PathBuf::from(JAVA_BIN))
}

/// Рантайм из сборки — как запасной путь, когда своей JVM на машине нет.
async fn staged(ctx: &BootstrapCtx<'_>) -> Result<PathBuf> {
    let files = crate::db::base_build_files(&ctx.state.db, ctx.base_build_id).await?;

    // Только своя платформа. Без этого фильтра сюда попадали рантаймы всех пяти
    // платформ, а `bin/java*` перезаписывался последним по алфавиту — то есть
    // `windows-x86_64/bin/java.exe`, который на Linux не запускается.
    let host = Platform::host();
    let prefix = format!("runtime/{}/", host.tag());
    let java_files: Vec<_> = files
        .iter()
        .filter(|f| f.kind == "java" && f.path.starts_with(&prefix))
        .collect();
    if java_files.is_empty() {
        bail!(
            "нет java для процессоров: в PATH и JAVA_HOME её нет, рантайма под {} в сборке тоже нет",
            host.tag()
        );
    }

    // Java на macOS/Linux не является одним самодостаточным бинарником: bin/java
    // ищет соседние lib/*.dylib/*.so через rpath. Поэтому восстанавливаем весь
    // runtime tree из FileStore во временную директорию.
    let runtime_dir = ctx
        .state
        .config
        .data_dir
        .join("tmp")
        .join(format!("java-runtime-{}", ctx.base_build_id));
    let _ = tokio::fs::remove_dir_all(&runtime_dir).await;
    tokio::fs::create_dir_all(&runtime_dir).await?;

    let mut java_bin = None;
    for f in java_files {
        let Some(rel) = f.path.strip_prefix(&prefix) else {
            continue;
        };
        let dst = runtime_dir.join(rel);
        if let Some(parent) = dst.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(ctx.state.files.path_for(&f.sha1), &dst)
            .await
            .with_context(|| format!("stage java {}", f.path))?;

        if f.path.ends_with("/bin/java") || f.path.ends_with("/bin/java.exe") {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = tokio::fs::metadata(&dst).await?.permissions();
                perms.set_mode(0o755);
                tokio::fs::set_permissions(&dst, perms).await?;
            }
            java_bin = Some(dst);
        }
    }

    let java_bin = java_bin.ok_or_else(|| anyhow!("java runtime скачан, но bin/java не найден"))?;
    // Абсолютный путь обязателен: процессор запускается из другой директории.
    Ok(tokio::fs::canonicalize(&java_bin).await.unwrap_or(java_bin))
}
