//! Сборка лаунчера из git-тега: checkout → cargo build → ed25519-подпись → FileStore.

pub mod dispatch;
pub mod github_watcher;

use crate::files::sha256_bytes;
use crate::state::AppState;
use anyhow::{anyhow, Context, Result};
use base64::Engine;
use std::path::PathBuf;
use uuid::Uuid;

/// Запустить сборку в фоне, вернуть id задачи.
pub async fn start_build(state: &AppState, tag: &str) -> Result<Uuid> {
    let repo = state
        .config
        .launcher_repo_path
        .clone()
        .ok_or_else(|| anyhow!("NORO_LAUNCHER_REPO не задан"))?;

    let job_id: Uuid = sqlx::query_scalar(
        "INSERT INTO launcher_build_jobs (github_tag, status) VALUES ($1, 'pending') RETURNING id",
    )
    .bind(tag)
    .fetch_one(&state.db)
    .await?;

    let state = state.clone();
    let tag = tag.to_string();
    tokio::spawn(async move {
        if let Err(e) = run_build(&state, job_id, &repo, &tag).await {
            let _ = append_log(&state, job_id, &format!("\nОШИБКА: {e:#}")).await;
            let _ = set_status(&state, job_id, "failed").await;
        }
    });

    Ok(job_id)
}

async fn run_build(state: &AppState, job_id: Uuid, _repo_name: &PathBuf, tag: &str) -> Result<()> {
    trigger_and_wait(state, job_id, tag).await?;

    set_status(state, job_id, "downloading").await?;
    append_log(
        state,
        job_id,
        &format!("запрос релиза {tag} из GitHub...\n"),
    )
    .await?;

    let repo_str = state
        .config
        .github_repo
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("NORO_GITHUB_REPO не задан — откуда брать релиз, неизвестно"))?;
    let url = format!("https://api.github.com/repos/{repo_str}/releases/tags/{tag}");

    let mut req = state.http().get(&url).header("User-Agent", "noro-master");
    if let Some(tok) = &state.config.github_token {
        req = req.bearer_auth(tok);
    }

    let resp = req
        .send()
        .await
        .with_context(|| "ошибка запроса к GitHub API")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "GitHub API вернул статус: {} ({})",
            status,
            text
        ));
    }

    let release: serde_json::Value = resp.json().await?;
    let assets = release["assets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("В релизе нет ассетов"))?;

    if assets.is_empty() {
        return Err(anyhow::anyhow!("Ассеты для релиза {} не найдены", tag));
    }

    append_log(
        state,
        job_id,
        &format!("найдено {} ассетов\n", assets.len()),
    )
    .await?;

    let target_platforms = [
        ("x86_64-pc-windows-msvc", "windows-x86_64"),
        ("x86_64-unknown-linux-gnu", "linux-x86_64"),
        ("aarch64-unknown-linux-gnu", "linux-aarch64"),
        ("x86_64-apple-darwin", "macos-x86_64"),
        ("aarch64-apple-darwin", "macos-aarch64"),
    ];

    let mut saved_count = 0;

    for asset in assets {
        let name = asset["name"].as_str().unwrap_or_default();
        // Порядок проверок важен: имя core тоже начинается с "noro-launcher-",
        // и обратный порядок пометил бы core установщиком.
        let kind = if name.starts_with("noro-launcher-core-") {
            "core"
        } else if name.starts_with("noro-launcher-") {
            "bootstrapper"
        } else {
            continue;
        };

        let mut matched_platform = None;
        for (target, plat) in &target_platforms {
            if name.contains(target) {
                matched_platform = Some(*plat);
                break;
            }
        }

        let Some(platform) = matched_platform else {
            continue;
        };
        let asset_url = asset["url"].as_str().unwrap_or_default();

        append_log(state, job_id, &format!("скачивание {}...\n", name)).await?;

        // Скачивание через API с токеном (чтобы работало с приватными репозиториями)
        let mut dl_req = state
            .http()
            .get(asset_url)
            .header("User-Agent", "noro-master")
            .header("Accept", "application/octet-stream");

        if let Some(tok) = &state.config.github_token {
            dl_req = dl_req.bearer_auth(tok);
        }

        let resp = dl_req.send().await?;
        if !resp.status().is_success() {
            append_log(
                state,
                job_id,
                &format!("ошибка скачивания {}: {}\n", name, resp.status()),
            )
            .await?;
            continue;
        }

        let bytes = resp.bytes().await?;

        let sha256 = sha256_bytes(&bytes);
        let signature = base64::engine::general_purpose::STANDARD.encode(state.signer.sign(&bytes));
        let stored = state.files.put_bytes(&bytes).await?;

        let id = crate::db::insert_launcher_version(
            &state.db,
            tag,
            platform,
            &sha256,
            &stored.sha1,
            bytes.len() as i64,
            &signature,
            kind,
        )
        .await?;

        saved_count += 1;
        append_log(
            state,
            job_id,
            &format!(
                "сохранено: {kind} {platform}, id={id}, {} байт\n",
                bytes.len()
            ),
        )
        .await?;
    }

    if saved_count == 0 {
        return Err(anyhow::anyhow!(
            "Не удалось скачать ни один бинарник noro-launcher для известных платформ"
        ));
    }

    set_status(state, job_id, "done").await?;
    Ok(())
}

/// Просит GitHub собрать лаунчер и ждёт окончания сборки.
///
/// Без токена запускать нечем — тогда работаем по релизу, который собрали
/// руками. Раньше это был единственный режим, и ломать его не за чем.
async fn trigger_and_wait(state: &AppState, job_id: Uuid, tag: &str) -> Result<()> {
    set_status(state, job_id, "building").await?;

    if dispatch::release_exists(state, tag).await.unwrap_or(false) {
        append_log(
            state,
            job_id,
            &format!("релиз {tag} уже собран — беру его ассеты\n"),
        )
        .await?;
        return Ok(());
    }

    if !dispatch::trigger(state, tag, job_id).await? {
        append_log(
            state,
            job_id,
            "GITHUB_TOKEN не задан — сборка не запускается, беру готовый релиз\n",
        )
        .await?;
        return Ok(());
    }

    append_log(state, job_id, &format!("запущена сборка лаунчера {tag}\n")).await?;

    // Лог пишется по ходу опроса, иначе полчаса ожидания выглядят как зависание.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let pump = {
        let state = state.clone();
        tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                let _ = append_log(&state, job_id, &line).await;
            }
        })
    };

    let result = dispatch::wait(state, job_id, |line| {
        let _ = tx.send(line);
    })
    .await;
    drop(tx);
    let _ = pump.await;

    let url = result?;
    append_log(state, job_id, &format!("сборка успешна: {url}\n")).await?;
    Ok(())
}

async fn append_log(state: &AppState, job_id: Uuid, text: &str) -> Result<()> {
    sqlx::query("UPDATE launcher_build_jobs SET log = log || $2 WHERE id = $1")
        .bind(job_id)
        .bind(text)
        .execute(&state.db)
        .await?;
    Ok(())
}

async fn set_status(state: &AppState, job_id: Uuid, status: &str) -> Result<()> {
    sqlx::query("UPDATE launcher_build_jobs SET status = $2 WHERE id = $1")
        .bind(job_id)
        .bind(status)
        .execute(&state.db)
        .await?;
    Ok(())
}
