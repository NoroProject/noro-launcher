//! Конфигурация мастер-сервера из переменных окружения.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Адрес для прослушивания, например "0.0.0.0:8080".
    pub bind_addr: String,
    /// Публичный базовый URL мастера ("https://master.noro.gg"). Используется в
    /// URL файлов, Yggdrasil-манифесте и OAuth redirect.
    pub public_url: String,
    /// Строка подключения к PostgreSQL.
    pub database_url: String,
    /// Корень данных: FileStore, ключи, временные файлы.
    pub data_dir: PathBuf,

    /// Discord OAuth.
    pub discord_client_id: String,
    pub discord_client_secret: String,

    /// Опциональный ключ CurseForge API для импорта/поиска.
    pub curseforge_api_key: Option<String>,

    /// ed25519 приватный ключ (hex, 32 байта seed). Если пусто — dev-режим.
    pub signing_key_hex: Option<String>,

    /// GitHub-репозиторий лаунчера для launcher_builder: "owner/repo".
    pub github_repo: Option<String>,
    pub github_token: Option<String>,
    /// Ветка, из которой запускается workflow сборки лаунчера.
    pub github_ref: Option<String>,
    /// Путь к локальному чекауту репозитория лаунчера для сборки.
    pub launcher_repo_path: Option<PathBuf>,

    /// Опциональный публичный URL для отдачи файлов (CDN, S3, R2, MinIO).
    /// Если задан — `file_url()` возвращает `{files_cdn_url}/{sha1}` вместо
    /// `{public_url}/files/{sha1}`. Полезно при отдаче через S3 / CloudFlare R2.
    pub files_cdn_url: Option<String>,

    /// Конфигурация S3-совместимого хранилища для загрузки файлов.
    pub s3: Option<S3Config>,
}

/// Настройки S3-совместимого хранилища (AWS S3, Cloudflare R2, MinIO).
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

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // .env подхватывается вызывающим (main).
        let bind_addr = env_or("NORO_BIND", "0.0.0.0:8080");
        let public_url = env_or("NORO_PUBLIC_URL", "http://localhost:8080");
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/noro".to_string());
        let data_dir = PathBuf::from(env_or("NORO_DATA_DIR", "./data"));

        let discord_client_id = env_or("DISCORD_CLIENT_ID", "");
        let discord_client_secret = env_or("DISCORD_CLIENT_SECRET", "");

        Ok(Self {
            bind_addr,
            public_url: public_url.trim_end_matches('/').to_string(),
            database_url,
            data_dir,
            discord_client_id,
            discord_client_secret,
            curseforge_api_key: std::env::var("CURSEFORGE_API_KEY").ok(),
            signing_key_hex: std::env::var("NORO_SIGNING_KEY")
                .ok()
                .filter(|s| !s.is_empty()),
            github_repo: std::env::var("NORO_GITHUB_REPO").ok(),
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            github_ref: std::env::var("NORO_GITHUB_REF").ok(),
            launcher_repo_path: std::env::var("NORO_LAUNCHER_REPO").ok().map(PathBuf::from),
            files_cdn_url: std::env::var("NORO_FILES_CDN_URL")
                .ok()
                .map(|u| u.trim_end_matches('/').to_string()),
            s3: S3Config::from_env(),
        })
    }

    /// URL для скачивания файла по SHA1.
    /// Если задан `files_cdn_url` — использует его, иначе `/files/{sha1}` на мастере.
    pub fn file_url(&self, sha1: &str) -> String {
        if let Some(cdn) = &self.files_cdn_url {
            format!("{cdn}/{sha1}")
        } else {
            format!("{}/files/{}", self.public_url, sha1)
        }
    }

    /// URL скина по умолчанию. Отдаётся мастером, а не хранится у каждого
    /// пользователя: так его можно поменять в одном месте.
    pub fn default_skin_url(&self) -> String {
        format!("{}/api/textures/default-skin", self.public_url)
    }

    /// Discord OAuth redirect для callback'а сайта.
    pub fn discord_redirect_uri(&self) -> String {
        format!("{}/auth/discord/callback", self.public_url)
    }

    /// Discord OAuth redirect для callback'а лаунчера.
    pub fn discord_launcher_redirect_uri(&self) -> String {
        format!("{}/auth/discord/launcher/callback", self.public_url)
    }

    pub fn is_dev_signing(&self) -> bool {
        self.signing_key_hex.is_none()
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

impl S3Config {
    fn from_env() -> Option<Self> {
        let endpoint = std::env::var("NORO_S3_ENDPOINT").ok()?;
        let bucket = std::env::var("NORO_S3_BUCKET").ok()?;
        let access_key = std::env::var("NORO_S3_ACCESS_KEY").ok()?;
        let secret_key = std::env::var("NORO_S3_SECRET_KEY").ok()?;
        let public_url = std::env::var("NORO_S3_PUBLIC_URL")
            .unwrap_or_else(|_| format!("{endpoint}/{bucket}"))
            .trim_end_matches('/')
            .to_string();
        let region = env_or("NORO_S3_REGION", "auto");
        Some(Self {
            endpoint,
            bucket,
            region,
            access_key,
            secret_key,
            public_url,
        })
    }
}
