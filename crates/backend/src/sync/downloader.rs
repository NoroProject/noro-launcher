//! Parallel download pool: retries with backoff, byte-level progress.

use super::fetch::fetch_to_file;
use super::integrity::sha1_file;
use anyhow::{bail, Result};
use futures::stream::{self, StreamExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
    pub sha1: String,
    pub size: u64,
    pub executable: bool,
}

/// Network hiccups under load are normal; one 502 must not take the whole
/// stage down with it.
const MAX_ATTEMPTS: u32 = 4;
const BASE_BACKOFF_MS: u64 = 300;
/// Don't wake the UI on every chunk.
const PROGRESS_STEP: i64 = 512 * 1024;

/// `on_progress` gets the running byte total; `cancelled` aborts the pool.
pub async fn download_all(
    client: &reqwest::Client,
    tasks: Vec<DownloadTask>,
    concurrency: usize,
    on_progress: impl Fn(u64) + Send + Sync + 'static,
    cancelled: impl Fn() -> bool + Send + Sync + 'static,
) -> Result<()> {
    let done = Arc::new(AtomicI64::new(0));
    let reported = Arc::new(AtomicI64::new(0));
    let on_progress = Arc::new(on_progress);
    let cancelled = Arc::new(cancelled);

    let results = stream::iter(tasks.into_iter().map(|task| {
        let client = client.clone();
        let done = done.clone();
        let reported = reported.clone();
        let on_progress = on_progress.clone();
        let cancelled = cancelled.clone();
        async move {
            if cancelled() {
                bail!("cancelled");
            }
            let bytes = {
                let done = done.clone();
                let reported = reported.clone();
                let on_progress = on_progress.clone();
                move |delta: i64| {
                    let total = done.fetch_add(delta, Ordering::Relaxed) + delta;
                    let last = reported.load(Ordering::Relaxed);
                    if (total - last).abs() >= PROGRESS_STEP {
                        reported.store(total, Ordering::Relaxed);
                        on_progress(total.max(0) as u64);
                    }
                }
            };
            download_with_retry(&client, &task, &bytes, cancelled.as_ref()).await?;
            // File finished: report the exact total instead of waiting for
            // the next progress step.
            on_progress(done.load(Ordering::Relaxed).max(0) as u64);
            Ok::<_, anyhow::Error>(())
        }
    }))
    .buffer_unordered(concurrency)
    .collect::<Vec<_>>()
    .await;

    for r in results {
        r?;
    }
    Ok(())
}

async fn download_with_retry(
    client: &reqwest::Client,
    task: &DownloadTask,
    on_bytes: &(dyn Fn(i64) + Send + Sync),
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<()> {
    let mut attempt = 1;
    loop {
        let result = fetch_to_file(client, &task.url, &task.dest, &task.sha1, on_bytes).await;
        match result {
            Ok(()) => break,
            Err(e) if attempt >= MAX_ATTEMPTS || cancelled() => {
                return Err(e.context(format!("download of {} failed", task.url)));
            }
            Err(e) => {
                let delay = BASE_BACKOFF_MS * 2u64.pow(attempt - 1) + jitter_ms(BASE_BACKOFF_MS);
                tracing::warn!(
                    url = %task.url, attempt, error = %e,
                    "retrying download in {delay}ms"
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
                attempt += 1;
            }
        }
    }

    // The java binary and natives need the exec bit on unix.
    #[cfg(unix)]
    if task.executable {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&task.dest).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&task.dest, perms).await?;
    }
    Ok(())
}

/// Spreads retries out so a hundred launchers don't come back in one wave.
fn jitter_ms(max: u64) -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::from(d.subsec_nanos()) % max.max(1))
        .unwrap_or(0)
}

pub async fn needs_download(
    dest: &PathBuf,
    expected_size: u64,
    expected_sha1: &str,
    verify_hash: bool,
) -> bool {
    let Ok(meta) = tokio::fs::metadata(dest).await else {
        return true;
    };
    if meta.len() != expected_size {
        return true;
    }
    if verify_hash {
        match sha1_file(dest).await {
            Ok(actual) => !actual.eq_ignore_ascii_case(expected_sha1),
            Err(_) => true,
        }
    } else {
        // Size matched. Artifacts are immutable, so that is good enough.
        false
    }
}
