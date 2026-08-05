//! Bootstrapper лаунчера: проверка обновлений, скачивание core-бинарника, запуск.
//!
//! Этот файл НИКОГДА не обновляется после первой установки — это даёт
//! накопление SmartScreen-репутации на Windows.
//! Вся логика лаунчера живёт в core-бинарнике (`noro-launcher-core`),
//! который обновляется автоматически в `AppData/noro-launcher/`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod verify;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("noro-launcher");
    let _ = std::fs::create_dir_all(&app_dir);

    let core_path = app_dir.join(core_binary_name());

    // Проверяем подпись на КАЖДОМ запуске, а не только при скачивании: иначе
    // всё, что сумеет записать в AppData, исполнялось бы вечно.
    if core_path.exists() {
        match verify::verify_installed(&core_path) {
            Ok(()) => return run_core(&core_path),
            Err(e) => {
                eprintln!("установленный лаунчер не прошёл проверку подписи: {e:#}");
                eprintln!("он будет скачан заново");
                verify::discard(&core_path);
            }
        }
    }

    // Сюда попадаем и на первом запуске, и когда установленный core забракован.
    eprintln!("скачивание лаунчера...");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    match rt.block_on(download_core(&app_dir, &core_path)) {
        Ok(()) => run_core(&core_path),
        Err(e) => {
            eprintln!("ошибка скачивания: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn core_binary_name() -> &'static str {
    if cfg!(windows) {
        "noro-launcher-core.exe"
    } else {
        "noro-launcher-core"
    }
}

fn run_core(path: &std::path::Path) -> ExitCode {
    let status = std::process::Command::new(path)
        .args(std::env::args_os().skip(1))
        .status();
    match status {
        Ok(s) => {
            if s.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("не удалось запустить {}: {e}", path.display());
            ExitCode::FAILURE
        }
    }
}

async fn download_core(app_dir: &PathBuf, dest: &PathBuf) -> anyhow::Result<()> {
    let master_url = verify::master_url();
    let platform = current_platform();
    let url = format!(
        "{}/api/launcher/version?platform={platform}",
        master_url.trim_end_matches('/')
    );

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?.error_for_status()?;
    let info: serde_json::Value = resp.json().await?;

    if info.is_null() {
        anyhow::bail!("нет доступной версии лаунчера для {platform}");
    }

    let download_url = info["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("нет url в ответе"))?;
    let expected_sha = info["sha256"].as_str().unwrap_or_default();
    // Подпись обязательна: sha256 из этого же ответа ловит битую закачку, но не
    // подмену — кто подменит канал, подставит и файл, и его хеш.
    let signature = info["signature"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("мастер не отдал подпись лаунчера"))?;
    let version = info["version"]
        .as_str()
        .unwrap_or("unknown");

    eprintln!("скачивание версии {version}...");
    let resp = client.get(download_url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;

    // Проверка SHA256.
    use sha2::Digest;
    let hash = hex::encode(sha2::Sha256::digest(&bytes));
    if !expected_sha.is_empty() && !hash.eq_ignore_ascii_case(expected_sha) {
        anyhow::bail!("sha256 не совпал: ожидали {expected_sha}, получили {hash}");
    }

    verify::verify_bytes(&bytes, signature)
        .map_err(|e| anyhow::anyhow!("проверка подписи лаунчера не прошла: {e}"))?;

    // Записать файл.
    std::fs::write(dest, &bytes)?;
    verify::store(dest, signature)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(dest, perms)?;
    }

    // Сохранить версию.
    let version_file = app_dir.join("version");
    std::fs::write(version_file, version).ok();

    eprintln!("готово!");
    Ok(())
}

fn current_platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        ("windows", "x86_64") => "windows-x86_64",
        (os, arch) => {
            Box::leak(format!("{os}-{arch}").into_boxed_str())
        }
    }
}
