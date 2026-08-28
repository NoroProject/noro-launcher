//! Скачивание одного файла: потоком на диск, с докачкой и проверкой SHA1.
//!
//! Раньше файл целиком буферизовался в память перед записью — на JDK в 180 МБ
//! это давало соответствующий всплеск RSS, помноженный на параллелизм пула.

use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use reqwest::StatusCode;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Отчёт о записанных байтах. Знаковый: при сбросе негодного остатка прогресс
/// откатывается назад, иначе шкала уехала бы за 100%.
pub type BytesFn<'a> = &'a (dyn Fn(i64) + Send + Sync);

/// Имя временного файла. Суффиксом, а не через `with_extension`: тот заменяет
/// расширение, и соседи вроде `emotes/x.json` и `emotes/x.ogg` получали общий
/// `emotes/x.part` — качаясь параллельно, они писали друг поверх друга.
pub fn part_path(dest: &Path) -> PathBuf {
    let mut p = dest.as_os_str().to_owned();
    p.push(".part");
    PathBuf::from(p)
}

/// Скачать `url` в `dest`. Незавершённая загрузка живёт в соседнем `.part`,
/// поэтому обрыв связи стоит только недокачанного хвоста, а не всего файла.
pub async fn fetch_to_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
    on_bytes: BytesFn<'_>,
) -> Result<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
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

        // 416: остаток длиннее самого файла — он не от этой версии. Начинаем заново.
        if resp.status() == StatusCode::RANGE_NOT_SATISFIABLE && have > 0 {
            let _ = tokio::fs::remove_file(&part).await;
            on_bytes(-(have as i64));
            have = 0;
            continue;
        }
        let partial = resp.status() == StatusCode::PARTIAL_CONTENT;
        let resp = resp
            .error_for_status()
            .with_context(|| format!("статус {url}"))?;
        break (resp, partial);
    };

    let mut hasher = Sha1::new();
    let mut file = if resuming {
        // Сервер согласился дослать хвост — досчитаем хеш уже лежащего начала.
        hash_existing(&part, &mut hasher).await?;
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&part)
            .await
            .with_context(|| format!("открытие {}", part.display()))?
    } else {
        // Range проигнорирован (или не запрашивался) — пишем с нуля.
        if have > 0 {
            on_bytes(-(have as i64));
        }
        tokio::fs::File::create(&part)
            .await
            .with_context(|| format!("создание {}", part.display()))?
    };

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| format!("обрыв загрузки {url}"))?;
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        on_bytes(chunk.len() as i64);
    }
    file.flush().await?;
    drop(file);

    let actual = hex::encode(hasher.finalize());
    if !expected_sha1.is_empty() && !actual.eq_ignore_ascii_case(expected_sha1) {
        // Возможно, дописали поверх остатка от другой сборки. Сносим — повтор
        // начнёт с чистого листа, иначе докачка зациклилась бы на мусоре.
        let written = tokio::fs::metadata(&part)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        let _ = tokio::fs::remove_file(&part).await;
        on_bytes(-(written as i64));
        bail!("SHA1 mismatch для {url}: ожидали {expected_sha1}, получили {actual}");
    }

    tokio::fs::rename(&part, dest)
        .await
        .with_context(|| format!("переименование в {}", dest.display()))?;
    Ok(())
}

/// Прогнать уже скачанную часть через хешер, не держа её в памяти целиком.
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
