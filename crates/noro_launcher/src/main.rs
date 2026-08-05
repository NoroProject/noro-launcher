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
        .join(schema::launcher_dir_name());
    let _ = std::fs::create_dir_all(&app_dir);

    let core_path = app_dir.join(core_binary_name());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    // Проверяем подпись на КАЖДОМ запуске, а не только при скачивании: иначе
    // всё, что сумеет записать в AppData, исполнялось бы вечно.
    if core_path.exists() {
        match verify::verify_installed(&core_path) {
            Ok(()) if !rt.block_on(update_pending(&app_dir)) => return run_core(&core_path),
            Ok(()) => eprintln!("на мастере лежит другая версия — обновляемся"),
            Err(e) => {
                eprintln!("установленный лаунчер не прошёл проверку подписи: {e:#}");
                eprintln!("он будет скачан заново");
                verify::discard(&core_path);
            }
        }
    }

    // Сюда попадаем на первом запуске, при забракованном core и при обновлении.
    eprintln!("скачивание лаунчера...");
    match rt.block_on(download_core(&app_dir, &core_path)) {
        Ok(()) => run_core(&core_path),
        Err(e) => {
            eprintln!("ошибка скачивания: {e:#}");
            ExitCode::FAILURE
        }
    }
}

/// Показать окно консоли, если его нет.
///
/// Релизный bootstrapper помечен `windows_subsystem = "windows"`, чтобы ярлык не
/// открывал чёрный квадрат на каждый запуск. Но когда идёт скачивание, показать
/// прогресс больше негде.
#[cfg(windows)]
fn show_console() {
    // SAFETY: вызов идёт из main до порождения потоков; повторный AllocConsole
    // просто вернёт ошибку, которая нам не важна.
    unsafe {
        windows_sys::Win32::System::Console::AllocConsole();
    }
}

#[cfg(not(windows))]
fn show_console() {}

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

/// Мастер раздаёт версию, отличную от установленной?
///
/// Обновлять core обязан именно bootstrapper. Сам core этого не сделает: он
/// показывает кнопку обновления в настройках, а те лежат за экраном входа — и
/// когда обновление нужно как раз для входа, круг не разрывается.
///
/// Сеть недоступна или мастер молчит — запускаем что есть: игру важнее открыть,
/// чем упереться в обновление.
async fn update_pending(app_dir: &PathBuf) -> bool {
    let installed = std::fs::read_to_string(app_dir.join("version")).unwrap_or_default();
    let installed = installed.trim();
    if installed.is_empty() {
        return false;
    }
    let url = format!(
        "{}/api/launcher/version?platform={}",
        verify::master_url().trim_end_matches('/'),
        current_platform()
    );
    let Ok(resp) = reqwest::Client::new().get(&url).send().await else {
        return false;
    };
    let Ok(info) = resp.json::<serde_json::Value>().await else {
        return false;
    };
    info["version"].as_str().is_some_and(|remote| remote != installed)
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

    // На Windows релизная сборка идёт без консоли, поэтому весь вывод уходил в
    // никуда: игрок запускал ярлык и минуту смотрел в пустой рабочий стол, пока
    // качались пятнадцать мегабайт. Консоль открываем только здесь — когда
    // действительно есть что показать.
    show_console();
    eprintln!("скачивание версии {version}...");
    let mut resp = client.get(download_url).send().await?.error_for_status()?;

    // Читаем по кускам ради прогресса: reqwest отдаёт их сам, без futures.
    let total = resp.content_length().unwrap_or(0);
    let mut bytes: Vec<u8> = Vec::with_capacity(total as usize);
    let mut shown = 0u64;
    while let Some(chunk) = resp.chunk().await? {
        bytes.extend_from_slice(&chunk);
        let done = bytes.len() as u64;
        // Печатаем раз в пять процентов, иначе строка мельтешит.
        if total > 0 && done * 20 / total > shown {
            shown = done * 20 / total;
            eprintln!(
                "  {}% ({:.1} из {:.1} МБ)",
                done * 100 / total,
                done as f64 / 1_048_576.0,
                total as f64 / 1_048_576.0
            );
        }
    }

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
