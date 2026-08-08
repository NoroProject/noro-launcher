//! Запуск workflow сборки лаунчера в GitHub Actions и ожидание результата.
//!
//! `workflow_dispatch` не возвращает id запуска, поэтому запуск помечается
//! id задачи: workflow подставляет его в `run-name`, а мастер ищет по нему свой
//! run среди последних. Совпадение по одному лишь тегу не годится — двух сборок
//! одного тега достаточно, чтобы мастер начал ждать чужую.

use crate::state::AppState;
use anyhow::{anyhow, Context, Result};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

const WORKFLOW: &str = "release-launcher.yml";
const POLL_EVERY: Duration = Duration::from_secs(15);
/// Полная матрица из пяти таргетов идёт около получаса; час — запас на очередь.
const MAX_WAIT: Duration = Duration::from_secs(60 * 60);

fn api(state: &AppState, path: &str) -> String {
    let repo = state
        .config
        .github_repo
        .as_deref()
        .unwrap_or("NexBitstd/NoroLauncher");
    format!("https://api.github.com/repos/{repo}/{path}")
}

fn token(state: &AppState) -> Option<&str> {
    state.config.github_token.as_deref()
}

/// Просит GitHub собрать лаунчер. Возвращает `false`, если токена нет и
/// запускать нечем — тогда вызывающий работает по уже готовому релизу.
pub async fn trigger(state: &AppState, tag: &str, job_id: Uuid) -> Result<bool> {
    let Some(tok) = token(state) else {
        return Ok(false);
    };
    // Ветка по умолчанию в этом репозитории — master, не main. Переопределяется
    // через NORO_GITHUB_REF, если сборку понадобится запускать с другой.
    let branch = state.config.github_ref.as_deref().unwrap_or("master");
    let resp = state
        .http()
        .post(api(
            state,
            &format!("actions/workflows/{WORKFLOW}/dispatches"),
        ))
        .header("User-Agent", "noro-master")
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(tok)
        .json(&json!({
            "ref": branch,
            "inputs": { "tag": tag, "job_id": job_id.to_string() },
        }))
        .send()
        .await
        .context("не отправить workflow_dispatch")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("GitHub отклонил запуск сборки: {status} ({body})"));
    }
    Ok(true)
}

/// Есть ли уже готовый релиз с таким тегом.
///
/// Тег, отправленный в git, собирает лаунчер сам. Без этой проверки нажатие
/// «Build tag» в админке запускало бы вторую сборку тех же исходников и ждало
/// её полчаса — при том, что забирать ассеты можно сразу.
pub async fn release_exists(state: &AppState, tag: &str) -> Result<bool> {
    let mut req = state
        .http()
        .get(api(state, &format!("releases/tags/{tag}")))
        .header("User-Agent", "noro-master")
        .header("Accept", "application/vnd.github+json");
    if let Some(tok) = token(state) {
        req = req.bearer_auth(tok);
    }
    let resp = req.send().await.context("не проверить наличие релиза")?;
    if !resp.status().is_success() {
        return Ok(false);
    }
    let body: serde_json::Value = resp.json().await?;
    // Черновик без ассетов забирать нечего — считаем, что релиза ещё нет.
    Ok(body["assets"].as_array().is_some_and(|a| !a.is_empty()))
}

/// Ждёт завершения запуска, помеченного `job_id`. Возвращает ссылку на run.
pub async fn wait(state: &AppState, job_id: Uuid, mut log: impl FnMut(String)) -> Result<String> {
    let marker = format!("[{job_id}]");
    let deadline = tokio::time::Instant::now() + MAX_WAIT;
    let mut announced = false;

    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err(anyhow!(
                "сборка не завершилась за {} минут",
                MAX_WAIT.as_secs() / 60
            ));
        }

        match find_run(state, &marker).await? {
            None => {
                // Между dispatch и появлением run проходит несколько секунд.
                if !announced {
                    log("ожидание запуска сборки в GitHub Actions...\n".into());
                    announced = true;
                }
            }
            Some(run) => {
                let status = run["status"].as_str().unwrap_or("");
                let url = run["html_url"].as_str().unwrap_or_default().to_string();
                if status != "completed" {
                    log(format!("сборка {status}: {url}\n"));
                } else {
                    let conclusion = run["conclusion"].as_str().unwrap_or("unknown");
                    if conclusion == "success" {
                        return Ok(url);
                    }
                    return Err(anyhow!("сборка завершилась как {conclusion}: {url}"));
                }
            }
        }

        tokio::time::sleep(POLL_EVERY).await;
    }
}

async fn find_run(state: &AppState, marker: &str) -> Result<Option<serde_json::Value>> {
    let mut req = state
        .http()
        .get(api(
            state,
            &format!("actions/workflows/{WORKFLOW}/runs?event=workflow_dispatch&per_page=30"),
        ))
        .header("User-Agent", "noro-master")
        .header("Accept", "application/vnd.github+json");
    if let Some(tok) = token(state) {
        req = req.bearer_auth(tok);
    }

    let resp = req.send().await.context("не получить список запусков")?;
    if !resp.status().is_success() {
        // Сеть и лимиты GitHub моргают; это не повод ронять сборку.
        return Ok(None);
    }
    let body: serde_json::Value = resp.json().await?;
    let runs = body["workflow_runs"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    Ok(runs
        .into_iter()
        .find(|r| r["name"].as_str().is_some_and(|n| n.contains(marker))))
}
