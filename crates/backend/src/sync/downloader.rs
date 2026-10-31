// Over 150 lines: the pool, its retries, the per-file trace and the
// up-to-date check that decides what the pool gets.
//! Parallel download pool: retries with backoff, byte-level progress.

use super::fetch::fetch_to_file;
use super::hash_cache::HashCache;
use super::trace::{FetchStats, FileRecord, SyncTrace};
use crate::http::HttpClientPool;
use anyhow::{bail, Result};
use futures::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
    pub sha1: String,
    pub size: u64,
    pub executable: bool,
}

/// The connections a sync downloads over, and the trace it reports into.
#[derive(Clone)]
pub struct Net {
    pub pool: HttpClientPool,
    pub trace: Arc<SyncTrace>,
}

impl Net {
    /// One connection and a fresh trace — for tests and one-off fetches.
    pub fn plain(client: reqwest::Client) -> Self {
        Self {
            pool: client.into(),
            trace: SyncTrace::new(1),
        }
    }
}

/// Network hiccups under load are normal; one 502 must not take the whole
/// stage down with it.
const MAX_ATTEMPTS: u32 = 4;
const BASE_BACKOFF_MS: u64 = 300;
/// Don't wake the UI on every chunk.
const PROGRESS_STEP: i64 = 512 * 1024;

/// `on_progress` gets the running byte total; `cancelled` aborts the pool.
pub async fn download_all(
    net: &Net,
    stage: &'static str,
    tasks: Vec<DownloadTask>,
    concurrency: usize,
    on_progress: impl Fn(u64) + Send + Sync + 'static,
    cancelled: impl Fn() -> bool + Send + Sync + 'static,
) -> Result<()> {
    let done = Arc::new(AtomicI64::new(0));
    let reported = Arc::new(AtomicI64::new(0));
    let on_progress = Arc::new(on_progress);
    let cancelled = Arc::new(cancelled);
    let stage_start = Instant::now();
    net.trace.expect_files(tasks.len(), concurrency);

    let results = stream::iter(tasks.into_iter().map(|task| {
        let net = net.clone();
        let done = done.clone();
        let reported = reported.clone();
        let on_progress = on_progress.clone();
        let cancelled = cancelled.clone();
        async move {
            if cancelled() {
                bail!("cancelled");
            }
            // The stream builds this future only when a slot frees up, so the
            // time until now is time spent queued behind other files.
            let queued = stage_start.elapsed();
            let bytes = {
                let done = done.clone();
                let reported = reported.clone();
                let on_progress = on_progress.clone();
                let trace = net.trace.clone();
                move |delta: i64| {
                    trace.bytes(delta);
                    let total = done.fetch_add(delta, Ordering::Relaxed) + delta;
                    let last = reported.load(Ordering::Relaxed);
                    if (total - last).abs() >= PROGRESS_STEP {
                        reported.store(total, Ordering::Relaxed);
                        on_progress(total.max(0) as u64);
                    }
                }
            };
            let started = Instant::now();
            net.trace.started_file();
            let result = download_with_retry(&net, &task, &bytes, cancelled.as_ref()).await;
            let (conn, attempts, stats) = match &result {
                Ok(r) => r.clone(),
                Err(_) => (0, MAX_ATTEMPTS, FetchStats::default()),
            };
            net.trace.finished_file(FileRecord {
                url: task.url.clone(),
                stage: stage.to_string(),
                size: task.size,
                conn,
                queued_ms: queued.as_millis() as u64,
                ttfb_ms: stats.ttfb.as_millis() as u64,
                net_ms: stats.net.as_millis() as u64,
                disk_ms: stats.disk.as_millis() as u64,
                hash_ms: stats.hash.as_millis() as u64,
                total_ms: started.elapsed().as_millis() as u64,
                attempts,
                http: stats.http,
                cache: stats.cache,
                segments: stats.segments,
            });
            result?;
            // File finished: report the exact total instead of waiting for
            // the next progress step.
            on_progress(done.load(Ordering::Relaxed).max(0) as u64);
            Ok::<_, anyhow::Error>(())
        }
    }))
    .buffer_unordered(concurrency);

    // The first failure ends the stage: dropping the stream cancels what is
    // still in flight. Collecting first meant waiting for thousands of other
    // files before the player learned the sync had already failed.
    futures::pin_mut!(results);
    while let Some(r) = results.next().await {
        r?;
    }
    Ok(())
}

/// (connection, attempts, timings of the attempt that succeeded).
async fn download_with_retry(
    net: &Net,
    task: &DownloadTask,
    on_bytes: &(dyn Fn(i64) + Send + Sync),
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<(usize, u32, FetchStats)> {
    let mut attempt = 1;
    let (conn, stats) = loop {
        let result = fetch_once(net, task, on_bytes, cancelled).await;
        match result {
            Ok(done) => break done,
            Err(e) if attempt >= MAX_ATTEMPTS || cancelled() => {
                return Err(e.context(format!("download of {} failed", task.url)));
            }
            Err(e) => {
                net.trace.retry();
                let delay = BASE_BACKOFF_MS * 2u64.pow(attempt - 1) + jitter_ms(BASE_BACKOFF_MS);
                tracing::warn!(
                    url = %task.url, attempt, error = %format!("{e:#}"),
                    "retrying download in {delay}ms"
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
                attempt += 1;
            }
        }
    };

    // The java binary and natives need the exec bit on unix.
    #[cfg(unix)]
    if task.executable {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&task.dest).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&task.dest, perms).await?;
    }
    Ok((conn, attempt, stats))
}

/// Large files go out as parallel ranges over the whole pool; the rest, and
/// anything the server won't serve in ranges, as one stream.
async fn fetch_once(
    net: &Net,
    task: &DownloadTask,
    on_bytes: &(dyn Fn(i64) + Send + Sync),
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<(usize, FetchStats)> {
    if task.size >= super::segmented::MIN_SIZE && net.pool.size() > 1 {
        let split = super::segmented::fetch(
            &net.pool, &task.url, &task.dest, &task.sha1, task.size, on_bytes, cancelled,
        )
        .await?;
        if let Some(stats) = split {
            // One past the last index: the report reads it as "all of them".
            return Ok((net.pool.size(), stats));
        }
    }
    let (conn, client) = net.pool.pick();
    let stats = fetch_to_file(
        client, &task.url, &task.dest, &task.sha1, on_bytes, cancelled,
    )
    .await?;
    Ok((conn, stats))
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
    dest: &Path,
    expected_size: u64,
    expected_sha1: &str,
    verify_hash: bool,
    cache: &HashCache,
) -> bool {
    let Ok(meta) = tokio::fs::metadata(dest).await else {
        return true;
    };
    if meta.len() != expected_size {
        return true;
    }
    if verify_hash {
        match cache.sha1_of(dest).await {
            Some(actual) => !actual.eq_ignore_ascii_case(expected_sha1),
            None => true,
        }
    } else {
        // Size matched. Artifacts are immutable, so that is good enough.
        false
    }
}

#[cfg(test)]
#[path = "downloader_tests.rs"]
mod tests;
