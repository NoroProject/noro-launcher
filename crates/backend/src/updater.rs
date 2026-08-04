//! Самообновление лаунчера: скачивание, проверка sha256 + ed25519, замена бинарника.

use crate::directories::LauncherDirectories;
use crate::sync::integrity::sha256_hex;
use anyhow::{bail, Context, Result};
use schema::LauncherVersion;

/// Скачать и установить обновление. После успеха нужно вызвать [`restart`].
pub async fn install_update(
    client: &reqwest::Client,
    dirs: &LauncherDirectories,
    version: &LauncherVersion,
    on_progress: impl Fn(u64, u64),
) -> Result<std::path::PathBuf> {
    tokio::fs::create_dir_all(dirs.updates()).await.ok();

    // Скачать.
    let resp = client.get(&version.url).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let mut bytes = Vec::with_capacity(total as usize);
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        bytes.extend_from_slice(&chunk);
        on_progress(bytes.len() as u64, total);
    }

    // Проверки целостности и подписи.
    let actual = sha256_hex(&bytes);
    if !actual.eq_ignore_ascii_case(&version.sha256) {
        bail!(
            "sha256 не совпал: ожидали {}, получили {actual}",
            version.sha256
        );
    }
    if !crate::signing::verify_bytes(&bytes, &version.signature) {
        bail!("подпись бинарника недействительна");
    }

    // Записать новый бинарник.
    let new_path = dirs
        .updates()
        .join(format!("noro-launcher-{}", version.version));
    tokio::fs::write(&new_path, &bytes).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&new_path).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&new_path, perms).await?;
    }

    // Заменить текущий исполняемый файл.
    let current = std::env::current_exe().context("определение текущего exe")?;
    swap_executable(&current, &new_path).await?;
    Ok(current)
}

/// Заменить `current` бинарник на `new`.
async fn swap_executable(current: &std::path::Path, new: &std::path::Path) -> Result<()> {
    #[cfg(unix)]
    {
        // На unix можно переименовать поверх запущенного бинарника.
        tokio::fs::rename(new, current)
            .await
            .context("замена бинарника")?;
    }
    #[cfg(windows)]
    {
        // На Windows запущенный exe нельзя перезаписать: переименуем старый.
        let old = current.with_extension("old");
        let _ = tokio::fs::remove_file(&old).await;
        tokio::fs::rename(current, &old)
            .await
            .context("переименование старого exe")?;
        tokio::fs::rename(new, current)
            .await
            .context("установка нового exe")?;
    }
    Ok(())
}

/// Перезапустить лаунчер из обновлённого бинарника и завершить текущий процесс.
pub fn restart(exe: &std::path::Path) -> ! {
    let _ = std::process::Command::new(exe).spawn();
    std::process::exit(0);
}
