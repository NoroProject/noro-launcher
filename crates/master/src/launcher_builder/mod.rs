//! Сборка лаунчера из git-тега: checkout → cargo build → ed25519-подпись → FileStore.

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

async fn run_build(state: &AppState, job_id: Uuid, repo: &PathBuf, tag: &str) -> Result<()> {
    set_status(state, job_id, "building").await?;
    append_log(state, job_id, &format!("сборка тега {tag}\n")).await?;

    // git fetch + checkout.
    run_logged(state, job_id, repo, "git", &["fetch", "--all", "--tags"]).await?;
    run_logged(state, job_id, repo, "git", &["checkout", tag]).await?;

    // cargo build.
    run_logged(
        state,
        job_id,
        repo,
        "cargo",
        &["build", "--release", "--bin", "noro-launcher"],
    )
    .await?;

    // Найти бинарник.
    let bin_name = if cfg!(windows) {
        "noro-launcher.exe"
    } else {
        "noro-launcher"
    };
    let bin_path = repo.join("target").join("release").join(bin_name);
    let bytes = tokio::fs::read(&bin_path)
        .await
        .with_context(|| format!("чтение бинарника {}", bin_path.display()))?;

    // Хеши и подпись.
    let sha256 = sha256_bytes(&bytes);
    let signature = base64::engine::general_purpose::STANDARD.encode(state.signer.sign(&bytes));
    let stored = state.files.put_bytes(&bytes).await?;

    let platform = schema::current_platform();
    let id = crate::db::insert_launcher_version(
        &state.db,
        tag,
        platform,
        &sha256,
        &stored.sha1,
        bytes.len() as i64,
        &signature,
    )
    .await?;

    append_log(
        state,
        job_id,
        &format!(
            "\nготово: версия {tag} ({platform}), id={id}, {} байт\n",
            bytes.len()
        ),
    )
    .await?;
    set_status(state, job_id, "done").await?;
    Ok(())
}

/// Запустить процесс и записать его вывод в лог задачи.
async fn run_logged(
    state: &AppState,
    job_id: Uuid,
    cwd: &PathBuf,
    cmd: &str,
    args: &[&str],
) -> Result<()> {
    append_log(state, job_id, &format!("$ {cmd} {}\n", args.join(" "))).await?;
    let output = tokio::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output()
        .await
        .with_context(|| format!("запуск {cmd}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    append_log(state, job_id, &format!("{stdout}{stderr}")).await?;
    if !output.status.success() {
        return Err(anyhow!(
            "{cmd} завершился с кодом {:?}",
            output.status.code()
        ));
    }
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
