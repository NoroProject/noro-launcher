//! Content-addressable хранилище: файл лежит по пути `files/{sha1[0:2]}/{sha1[2:4]}/{sha1}`.
//! Один и тот же контент хранится один раз независимо от числа сборок.

use anyhow::{Context, Result};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct FileStore {
    root: PathBuf,
}

/// Результат сохранения: хеш и размер.
pub struct StoredFile {
    pub sha1: String,
    pub size: u64,
}

impl FileStore {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            root: data_dir.join("files"),
        }
    }

    /// Абсолютный путь к файлу по sha1.
    pub fn path_for(&self, sha1: &str) -> PathBuf {
        self.root.join(&sha1[0..2]).join(&sha1[2..4]).join(sha1)
    }

    pub fn exists(&self, sha1: &str) -> bool {
        sha1.len() >= 4 && self.path_for(sha1).exists()
    }

    /// Сохранить байты, вернуть их sha1. Если уже есть — не перезаписывает.
    pub async fn put_bytes(&self, data: &[u8]) -> Result<StoredFile> {
        let sha1 = hex::encode(Sha1::digest(data));
        let dst = self.path_for(&sha1);
        if !dst.exists() {
            if let Some(parent) = dst.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            // Пишем во временный файл, затем атомарно переименовываем.
            let tmp = dst.with_extension("tmp");
            tokio::fs::write(&tmp, data).await?;
            tokio::fs::rename(&tmp, &dst).await?;
        }
        Ok(StoredFile {
            sha1,
            size: data.len() as u64,
        })
    }

    /// Сохранить из существующего файла (по пути), посчитав хеш потоково.
    pub async fn put_file(&self, src: &Path) -> Result<StoredFile> {
        let data = tokio::fs::read(src)
            .await
            .with_context(|| format!("чтение {}", src.display()))?;
        self.put_bytes(&data).await
    }

    /// Скачать по URL и сохранить, проверив SHA1 если задан.
    pub async fn put_url(
        &self,
        client: &reqwest::Client,
        url: &str,
        expected_sha1: Option<&str>,
    ) -> Result<StoredFile> {
        // Если ожидаемый хеш уже в сторе — скачивать не нужно.
        if let Some(sha1) = expected_sha1 {
            if self.exists(sha1) {
                let size = tokio::fs::metadata(self.path_for(sha1)).await?.len();
                return Ok(StoredFile {
                    sha1: sha1.to_string(),
                    size,
                });
            }
        }
        let resp = client
            .get(url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?
            .error_for_status()
            .with_context(|| format!("статус ответа для {url}"))?;
        let bytes = resp.bytes().await?;
        let stored = self.put_bytes(&bytes).await?;
        if let Some(expected) = expected_sha1 {
            if !expected.eq_ignore_ascii_case(&stored.sha1) {
                anyhow::bail!(
                    "SHA1 mismatch при скачивании {url}: ожидали {expected}, получили {}",
                    stored.sha1
                );
            }
        }
        Ok(stored)
    }

    /// Открыть файл для отдачи через HTTP.
    pub async fn open(&self, sha1: &str) -> Result<tokio::fs::File> {
        let path = self.path_for(sha1);
        tokio::fs::File::open(&path)
            .await
            .with_context(|| format!("файл {sha1} не найден в сторе"))
    }

    /// Размер хранилища (байт) — для статистики.
    pub async fn total_size(&self) -> u64 {
        let root = self.root.clone();
        tokio::task::spawn_blocking(move || {
            walkdir::WalkDir::new(&root)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum()
        })
        .await
        .unwrap_or(0)
    }
}

/// Посчитать SHA1 файла потоково (для импорта пользовательских файлов).
pub async fn sha1_file(path: &Path) -> Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buf = vec![0u8; 64 * 1024];
    use tokio::io::AsyncReadExt;
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Посчитать SHA256 (для ядер серверов и бинарников лаунчера).
pub fn sha256_bytes(data: &[u8]) -> String {
    use sha2::Sha256;
    hex::encode(Sha256::digest(data))
}

/// Вспомогательно: записать произвольные байты во временный файл и вернуть путь.
pub async fn write_temp(dir: &Path, name: &str, data: &[u8]) -> Result<PathBuf> {
    tokio::fs::create_dir_all(dir).await?;
    let path = dir.join(name);
    let mut f = tokio::fs::File::create(&path).await?;
    f.write_all(data).await?;
    f.flush().await?;
    Ok(path)
}
