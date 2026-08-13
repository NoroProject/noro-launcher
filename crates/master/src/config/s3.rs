//! Настройки S3-совместимого хранилища (AWS S3, Cloudflare R2, MinIO).

use super::{env_or, env_required};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    /// Публичный URL для генерации ссылок: `{public_url}/{sha1}`.
    pub public_url: String,
}

impl S3Config {
    /// `None` — хранилище просто не настроено, это штатный режим.
    ///
    /// Но если задана хоть одна переменная блока, остальные обязаны быть: связка
    /// из половины ключей раньше молча превращалась в «S3 выключен», и файлы
    /// уезжали на диск мастера вместо бакета.
    pub fn from_env() -> Result<Option<Self>> {
        const KEYS: [&str; 5] = [
            "NORO_S3_ENDPOINT",
            "NORO_S3_BUCKET",
            "NORO_S3_ACCESS_KEY",
            "NORO_S3_SECRET_KEY",
            "NORO_S3_PUBLIC_URL",
        ];
        if !KEYS.iter().any(|k| std::env::var(k).is_ok()) {
            return Ok(None);
        }

        Ok(Some(Self {
            endpoint: env_required("NORO_S3_ENDPOINT")?,
            bucket: env_required("NORO_S3_BUCKET")?,
            // "auto" — соглашение протокола (так требует R2), а не наша выдумка.
            region: env_or("NORO_S3_REGION", "auto"),
            access_key: env_required("NORO_S3_ACCESS_KEY")?,
            secret_key: env_required("NORO_S3_SECRET_KEY")?,
            // Раньше собирался как `{endpoint}/{bucket}`. У R2 и CDN он другой,
            // и такая ссылка вела в никуда — уже из подписанного манифеста.
            public_url: env_required("NORO_S3_PUBLIC_URL")?
                .trim_end_matches('/')
                .to_string(),
        }))
    }
}
