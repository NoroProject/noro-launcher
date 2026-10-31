// Over 150 lines: the split, the per-part fetch and the checks that make a
// file assembled from parts as trustworthy as one streamed whole.
//! A large file fetched as several byte ranges at once, one per connection.
//!
//! Throughput is capped per connection (see `http::HttpClientPool`), so a
//! single 180 MB runtime on one stream crawls at the single-connection rate
//! however fast the link is — and it is usually the last file left running.
//! Split across the pool it moves at the pool's rate.
//!
//! The server must answer every range with 206. If it ignores ranges, the
//! caller falls back to an ordinary download: `Ok(None)`.

use super::fetch::{part_path, BytesFn, CancelFn};
use super::trace::FetchStats;
use crate::http::HttpClientPool;
use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use reqwest::StatusCode;
use std::io::SeekFrom;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

/// Below this a file is not worth splitting: the extra requests cost more
/// round trips than they save.
pub const MIN_SIZE: u64 = 16 * 1024 * 1024;
/// No part smaller than this, so a 20 MB file on 16 connections isn't cut
/// into slivers.
const MIN_PART: u64 = 8 * 1024 * 1024;

/// Per-part timings, summed into the file's `FetchStats`.
#[derive(Default)]
struct PartStats {
    ttfb: Duration,
    net: Duration,
    disk: Duration,
}

enum PartError {
    /// 200 instead of 206: the server ignores ranges.
    NoRanges,
    Failed(anyhow::Error),
}

pub async fn fetch(
    pool: &HttpClientPool,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
    size: u64,
    on_bytes: BytesFn<'_>,
    cancelled: CancelFn<'_>,
) -> Result<Option<FetchStats>> {
    let parts = (size / MIN_PART).clamp(1, pool.size() as u64);
    fetch_split(
        pool,
        url,
        dest,
        expected_sha1,
        size,
        parts,
        on_bytes,
        cancelled,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn fetch_split(
    pool: &HttpClientPool,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
    size: u64,
    parts: u64,
    on_bytes: BytesFn<'_>,
    cancelled: CancelFn<'_>,
) -> Result<Option<FetchStats>> {
    if parts < 2 {
        return Ok(None);
    }
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let part = part_path(dest);
    // Ranges land in place, so the file is sized up front; a leftover partial
    // from a streamed attempt is useless here and gets truncated.
    let file = tokio::fs::File::create(&part)
        .await
        .with_context(|| format!("creating {}", part.display()))?;
    file.set_len(size).await?;
    drop(file);

    let written = AtomicI64::new(0);
    let count = |delta: i64| {
        written.fetch_add(delta, Ordering::Relaxed);
        on_bytes(delta);
    };
    let step = size.div_ceil(parts);
    let mut first: Option<Meta> = None;
    let jobs = (0..parts).map(|i| {
        let start = i * step;
        let end = (start + step).min(size) - 1;
        let (_, client) = pool.pick();
        fetch_part(client, url, &part, start, end, &count, cancelled)
    });
    let results = futures::future::join_all(jobs).await;

    let mut stats = FetchStats {
        segments: parts as u32,
        ..Default::default()
    };
    let mut outcome = Ok(());
    for r in results {
        match r {
            Ok((p, meta)) => {
                stats.ttfb = stats.ttfb.max(p.ttfb);
                stats.net += p.net;
                stats.disk += p.disk;
                first.get_or_insert(meta);
            }
            Err(e) if outcome.is_ok() => outcome = Err(e),
            Err(_) => {}
        }
    }
    if let Some((version, cache)) = first {
        stats.http = if version == reqwest::Version::HTTP_2 {
            "h2"
        } else {
            "h1.1"
        };
        stats.cache = cache;
    }

    if let Err(e) = outcome {
        // Roll the bar back: the next attempt counts these bytes again.
        on_bytes(-written.load(Ordering::Relaxed));
        let _ = tokio::fs::remove_file(&part).await;
        return match e {
            PartError::NoRanges => Ok(None),
            PartError::Failed(e) => Err(e),
        };
    }

    let t = Instant::now();
    let actual = super::integrity::sha1_file(&part).await?;
    stats.hash = t.elapsed();
    if !expected_sha1.is_empty() && !actual.eq_ignore_ascii_case(expected_sha1) {
        on_bytes(-written.load(Ordering::Relaxed));
        let _ = tokio::fs::remove_file(&part).await;
        bail!("SHA1 mismatch for {url}: expected {expected_sha1}, got {actual}");
    }
    let t = Instant::now();
    replace(&part, dest).await?;
    stats.disk += t.elapsed();
    Ok(Some(stats))
}

/// HTTP version and `cf-cache-status` of a part's response.
type Meta = (reqwest::Version, Option<String>);

async fn fetch_part(
    client: &reqwest::Client,
    url: &str,
    part: &Path,
    start: u64,
    end: u64,
    on_bytes: &(dyn Fn(i64) + Sync),
    cancelled: CancelFn<'_>,
) -> std::result::Result<(PartStats, Meta), PartError> {
    let fail = |e: anyhow::Error| PartError::Failed(e);
    let mut stats = PartStats::default();
    let sent = Instant::now();
    let resp = client
        .get(url)
        .header(reqwest::header::RANGE, format!("bytes={start}-{end}"))
        .send()
        .await
        .with_context(|| format!("GET {url}"))
        .map_err(fail)?;
    stats.ttfb = sent.elapsed();
    if resp.status() == StatusCode::OK {
        return Err(PartError::NoRanges);
    }
    let resp = resp
        .error_for_status()
        .with_context(|| format!("bad status from {url}"))
        .map_err(fail)?;
    if resp.status() != StatusCode::PARTIAL_CONTENT {
        return Err(PartError::NoRanges);
    }
    let meta = (
        resp.version(),
        resp.headers()
            .get("cf-cache-status")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
    );

    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(part)
        .await
        .map_err(|e| fail(e.into()))?;
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|e| fail(e.into()))?;
    let mut writer = tokio::io::BufWriter::with_capacity(256 * 1024, file);
    let mut stream = resp.bytes_stream();
    let mut got = 0u64;
    loop {
        let t = Instant::now();
        let Some(chunk) = stream.next().await else {
            break;
        };
        stats.net += t.elapsed();
        if cancelled() {
            return Err(fail(anyhow::anyhow!("cancelled")));
        }
        let chunk = chunk
            .with_context(|| format!("download from {url} interrupted"))
            .map_err(fail)?;
        let t = Instant::now();
        writer.write_all(&chunk).await.map_err(|e| fail(e.into()))?;
        stats.disk += t.elapsed();
        got += chunk.len() as u64;
        on_bytes(chunk.len() as i64);
    }
    let t = Instant::now();
    writer.flush().await.map_err(|e| fail(e.into()))?;
    stats.disk += t.elapsed();
    if got != end - start + 1 {
        return Err(fail(anyhow::anyhow!(
            "range {start}-{end} of {url} came back with {got} bytes"
        )));
    }
    Ok((stats, meta))
}

/// Move a finished `.part` over the destination.
pub(super) async fn replace(part: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        let _ = tokio::fs::remove_file(dest).await;
    }
    if let Err(e) = tokio::fs::rename(part, dest).await {
        // Another pass placed the same file meanwhile; ours is redundant.
        if let Ok(m) = tokio::fs::metadata(dest).await {
            if m.len() > 0 {
                let _ = tokio::fs::remove_file(part).await;
                return Ok(());
            }
        }
        return Err(e).with_context(|| format!("renaming to {}", dest.display()));
    }
    Ok(())
}

#[cfg(test)]
#[path = "segmented_tests.rs"]
mod tests;
