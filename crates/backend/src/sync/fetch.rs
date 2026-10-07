// File exceeds 150 lines: single-file fetcher with memory fast-path and resumable streaming.
//! One file: streamed to disk, resumable, SHA1-checked.
//!
//! Streaming rather than buffering matters here — a 180 MB JDK held in memory,
//! multiplied by the pool's parallelism, is a lot of RSS.

use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use reqwest::StatusCode;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Bytes written. Signed, because discarding a bad partial has to roll the
/// progress back — otherwise the bar runs past 100%.
pub type BytesFn<'a> = &'a (dyn Fn(i64) + Send + Sync);

/// Polled between chunks, so a cancel stops a 200 MB download mid-file rather
/// than after it. The partial stays for the next attempt to resume.
pub type CancelFn<'a> = &'a (dyn Fn() -> bool + Send + Sync);

/// Appends `.part` instead of using `with_extension`, which replaces the
/// extension: `emotes/x.json` and `emotes/x.ogg` would share one partial file
/// and, downloading in parallel, write over each other.
pub fn part_path(dest: &Path) -> PathBuf {
    let mut p = dest.as_os_str().to_owned();
    p.push(".part");
    PathBuf::from(p)
}

/// Direct in-memory fetch for small files (≤ 512 KB, e.g. configs and tiny assets).
/// Bypasses .part files, range checks, and rename syscalls completely.
pub async fn fetch_small_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
    on_bytes: BytesFn<'_>,
    cancelled: CancelFn<'_>,
) -> Result<()> {
    if cancelled() {
        bail!("cancelled");
    }
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;
    let resp = resp
        .error_for_status()
        .with_context(|| format!("bad status from {url}"))?;
    if cancelled() {
        bail!("cancelled");
    }
    let bytes = resp.bytes().await?;
    if cancelled() {
        bail!("cancelled");
    }
    let actual = hex::encode(Sha1::digest(&bytes));
    if !expected_sha1.is_empty() && !actual.eq_ignore_ascii_case(expected_sha1) {
        bail!("SHA1 mismatch for {url}: expected {expected_sha1}, got {actual}");
    }
    if let Err(e) = tokio::fs::write(dest, &bytes).await {
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
            tokio::fs::write(dest, &bytes).await?;
        } else {
            return Err(e.into());
        }
    }
    on_bytes(bytes.len() as i64);
    Ok(())
}

/// Download `url` to `dest`. Work in progress lives in a neighbouring `.part`,
/// so a dropped connection costs the remaining tail, not the whole file.
pub async fn fetch_to_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
    on_bytes: BytesFn<'_>,
    cancelled: CancelFn<'_>,
) -> Result<()> {
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    let part = part_path(dest);
    let mut have = tokio::fs::metadata(&part)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    let (resp, resuming) = loop {
        let mut req = client.get(url);
        if have > 0 {
            req = req.header(reqwest::header::RANGE, format!("bytes={have}-"));
        }
        let resp = req.send().await.with_context(|| format!("GET {url}"))?;

        // 416: the partial is longer than the file, so it's left over from
        // another version. Start again.
        if resp.status() == StatusCode::RANGE_NOT_SATISFIABLE && have > 0 {
            let _ = tokio::fs::remove_file(&part).await;
            on_bytes(-(have as i64));
            have = 0;
            continue;
        }
        let partial = resp.status() == StatusCode::PARTIAL_CONTENT;
        let resp = resp
            .error_for_status()
            .with_context(|| format!("bad status from {url}"))?;
        break (resp, partial);
    };

    // Small files (e.g. thousands of quest configs) write directly in one go
    // without intermediate .part and rename syscalls.
    let len = resp.content_length().unwrap_or(0);
    if have == 0 && !resuming && len > 0 && len <= 512 * 1024 {
        if cancelled() {
            bail!("cancelled");
        }
        let bytes = resp.bytes().await?;
        if cancelled() {
            bail!("cancelled");
        }
        let actual = hex::encode(Sha1::digest(&bytes));
        if !expected_sha1.is_empty() && !actual.eq_ignore_ascii_case(expected_sha1) {
            bail!("SHA1 mismatch for {url}: expected {expected_sha1}, got {actual}");
        }
        tokio::fs::write(dest, &bytes).await?;
        on_bytes(bytes.len() as i64);
        return Ok(());
    }

    let mut hasher = Sha1::new();
    let file = if resuming {
        // The server is sending the tail, so the hash has to cover what's
        // already on disk before it.
        hash_existing(&part, &mut hasher).await?;
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&part)
            .await
            .with_context(|| format!("opening {}", part.display()))?
    } else {
        // Range ignored, or never asked for. Write from scratch.
        if have > 0 {
            on_bytes(-(have as i64));
        }
        tokio::fs::File::create(&part)
            .await
            .with_context(|| format!("creating {}", part.display()))?
    };

    let mut writer = tokio::io::BufWriter::with_capacity(128 * 1024, file);
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if cancelled() {
            writer.flush().await?;
            bail!("cancelled");
        }
        let chunk = chunk.with_context(|| format!("download from {url} interrupted"))?;
        hasher.update(&chunk);
        writer.write_all(&chunk).await?;
        on_bytes(chunk.len() as i64);
    }
    writer.flush().await?;
    drop(writer);

    let actual = hex::encode(hasher.finalize());
    if !expected_sha1.is_empty() && !actual.eq_ignore_ascii_case(expected_sha1) {
        // We may have appended onto a partial from another build. Remove it, or
        // the retry resumes from the same garbage and never converges.
        let written = tokio::fs::metadata(&part)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        let _ = tokio::fs::remove_file(&part).await;
        on_bytes(-(written as i64));
        bail!("SHA1 mismatch for {url}: expected {expected_sha1}, got {actual}");
    }

    if dest.exists() {
        let _ = tokio::fs::remove_file(dest).await;
    }
    if let Err(e) = tokio::fs::rename(&part, dest).await {
        if dest.exists() {
            if let Ok(m) = tokio::fs::metadata(dest).await {
                if m.len() > 0 {
                    let _ = tokio::fs::remove_file(&part).await;
                    return Ok(());
                }
            }
        }
        return Err(e).with_context(|| format!("renaming to {}", dest.display()));
    }
    Ok(())
}

async fn hash_existing(part: &Path, hasher: &mut Sha1) -> Result<()> {
    let mut file = tokio::fs::File::open(part).await?;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        hasher.update(&buf[..n]);
    }
}

#[cfg(test)]
#[path = "fetch_tests.rs"]
mod tests;
