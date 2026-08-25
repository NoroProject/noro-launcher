//! Самообновление лаунчера: скачивание, проверка sha256 + ed25519, установка в AppData/bin/.
//! Bootstrapper (.exe, который скачал пользователь) никогда не меняется — это даёт
//! накопление SmartScreen-репутации на Windows.

use crate::directories::LauncherDirectories;
use crate::sync::integrity::sha256_hex;
use anyhow::{bail, Context, Result};
use schema::LauncherVersion;
use std::path::PathBuf;

/// Имя основного бинарника лаунчера в каталоге `bin/`.
fn core_binary_name() -> &'static str {
    if cfg!(windows) {
        "noro-launcher-core.exe"
    } else {
        "noro-launcher-core"
    }
}

/// Путь к основному бинарнику лаунчера.
pub fn core_binary_path(dirs: &LauncherDirectories) -> PathBuf {
    dirs.root().join(core_binary_name())
}

/// Скачать и установить обновление в `AppData/bin/`. Возвращает путь к бинарнику.
pub async fn install_update(
    client: &reqwest::Client,
    dirs: &LauncherDirectories,
    version: &LauncherVersion,
    on_progress: impl Fn(u64, u64),
) -> Result<PathBuf> {
    tokio::fs::create_dir_all(dirs.root()).await.ok();

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

    // Записать основной бинарник в bin/.
    let dest = core_binary_path(dirs);

    // На Windows нельзя перезаписать запущенный exe — переименуем старый.
    #[cfg(windows)]
    if dest.exists() {
        let old = dest.with_extension("old");
        let _ = tokio::fs::remove_file(&old).await;
        let _ = tokio::fs::rename(&dest, &old).await;
    }

    tokio::fs::write(&dest, &bytes).await?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&dest).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&dest, perms).await?;
    }

    // Сохранить текущую версию.
    //
    // Не «по возможности»: по этому файлу решается, надо ли обновляться. Если
    // он не записался, лаунчер при каждом запуске считает себя устаревшим и
    // качает одно и то же обновление заново — молча и бесконечно.
    let version_file = dirs.root().join("version");
    tokio::fs::write(&version_file, &version.version)
        .await
        .with_context(|| format!("запись {}", version_file.display()))?;

    Ok(dest)
}

/// Перезапустить лаунчер (запускает основной бинарник из bin/).
pub fn restart(exe: &std::path::Path) -> ! {
    let _ = std::process::Command::new(exe).spawn();
    std::process::exit(0);
}
