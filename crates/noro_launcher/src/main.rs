//! Bootstrapper лаунчера: проверка обновлений, скачивание core-бинарника, запуск.
//!
//! Этот файл НИКОГДА не обновляется после первой установки — это даёт
//! накопление SmartScreen-репутации на Windows.
//! Вся логика лаунчера живёт в core-бинарнике (`noro-launcher-core`),
//! который обновляется автоматически в `AppData/noro-launcher/`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("noro-launcher");
    let _ = std::fs::create_dir_all(&app_dir);

    let core_path = app_dir.join(core_binary_name());

    // Если core-бинарник есть — просто запускаем.
    // Обновление проверит сам core (backend::check_launcher_update).
    if core_path.exists() {
        return run_core(&core_path);
    }

    // Первый запуск: скачиваем core с мастера.
    eprintln!("первый запуск — скачивание лаунчера...");
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
    let master_url = option_env!("NORO_MASTER_URL")
        .unwrap_or("http://127.0.0.1:8080");
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
    let expected_sha = info["sha256"]
        .as_str()
        .unwrap_or_default();
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

    // Записать файл.
    std::fs::write(dest, &bytes)?;

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
