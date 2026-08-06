//! Bootstrapper лаунчера: проверка обновлений, скачивание core-бинарника, запуск.
//!
//! Этот файл НИКОГДА не обновляется после первой установки — это даёт
//! накопление SmartScreen-репутации на Windows.
//! Вся логика лаунчера живёт в core-бинарнике (`noro-launcher-core`),
//! который обновляется автоматически в `AppData/noro-launcher/`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod splash;
mod verify;

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Посмотреть на окно загрузки, не дожидаясь настоящей закачки: править вид
    // иначе можно только вслепую, стирая скачанный core перед каждым запуском.
    // Только в отладочной сборке — в релизе такого крючка нет.
    #[cfg(debug_assertions)]
    if std::env::var_os("NORO_SPLASH_PREVIEW").is_some() {
        return splash_preview();
    }

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
    // Дальше — минуты закачки, поэтому показываем окно: GPUI забирает главный
    // поток себе, а скачивание уходит в фон и рапортует прогресс в общий слот.
    let (reporter, rx) = tokio::sync::mpsc::unbounded_channel();
    let work_dir = app_dir.clone();
    let work_core = core_path.clone();
    // На macOS `cx.quit()` вызывает `[NSApp terminate:]` и убивает процесс,
    // не возвращая управление из `run_with`. Поэтому запуск core делается
    // через `before_quit`, который гарантированно вызовется до `cx.quit()`.
    let success = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let success_w = success.clone();
    let launch_path = core_path.clone();
    let before_quit: Option<Box<dyn FnOnce() + Send + 'static>> =
        Some(Box::new(move || {
            if success.load(std::sync::atomic::Ordering::Acquire) {
                let _ = std::process::Command::new(&launch_path)
                    .args(std::env::args_os().skip(1))
                    .spawn();
            }
        }));

    let outcome = splash::run_with(
        rx,
        move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            let res = rt.block_on(download_core(&work_dir, &work_core, &reporter));
            if res.is_ok() {
                success_w.store(true, std::sync::atomic::Ordering::Release);
            }
            res
        },
        before_quit,
    );

    // Если `run_with` всё-таки вернёт управление (Linux/Windows) — запускаем
    // core отсюда. На macOS сюда не доходим.
    match outcome {
        Some(Ok(())) => run_core(&core_path),
        Some(Err(e)) => {
            eprintln!("ошибка скачивания: {e:#}");
            ExitCode::FAILURE
        }
        None => ExitCode::FAILURE,
    }
}

/// Крутит полосу по кругу, пока окно не закроют.
#[cfg(debug_assertions)]
fn splash_preview() -> ExitCode {
    let (reporter, rx) = tokio::sync::mpsc::unbounded_channel();
    splash::run_with(
        rx,
        move || {
            let stages = [
                ("Проверка версии…", 0u64),
                ("Загрузка launcher-v1.2.3", 15_358_608),
            ];
            loop {
                for (label, total) in stages {
                    for step in 0..=100 {
                        let _ = reporter.send(splash::Progress {
                            label: label.to_string(),
                            done: total / 100 * step,
                            total,
                        });
                        std::thread::sleep(std::time::Duration::from_millis(60));
                    }
                }
            }
        },
        None,
    );
    ExitCode::SUCCESS
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
    info["version"]
        .as_str()
        .is_some_and(|remote| remote != installed)
}

async fn download_core(
    app_dir: &PathBuf,
    dest: &PathBuf,
    report: &splash::Reporter,
) -> anyhow::Result<()> {
    let say = |label: &str, done: u64, total: u64| {
        let _ = report.send(splash::Progress {
            label: label.to_string(),
            done,
            total,
        });
    };
    say("Проверка версии…", 0, 0);
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
    let version = info["version"].as_str().unwrap_or("unknown");

    say(&format!("Загрузка {version}"), 0, 0);
    let mut resp = client.get(download_url).send().await?.error_for_status()?;

    // Читаем по кускам ради прогресса: reqwest отдаёт их сам, без futures.
    let total = resp.content_length().unwrap_or(0);
    let mut bytes: Vec<u8> = Vec::with_capacity(total as usize);
    // Отчитываемся раз в процент: кадр всё равно один, а сообщений было бы
    // столько же, сколько кусков в ответе.
    let mut reported = 0u64;
    while let Some(chunk) = resp.chunk().await? {
        bytes.extend_from_slice(&chunk);
        let done = bytes.len() as u64;
        if total > 0 && done * 100 / total > reported {
            reported = done * 100 / total;
            say(&format!("Загрузка {version}"), done, total);
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

    say("Готово", 1, 1);
    Ok(())
}

fn current_platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        ("windows", "x86_64") => "windows-x86_64",
        (os, arch) => Box::leak(format!("{os}-{arch}").into_boxed_str()),
    }
}
