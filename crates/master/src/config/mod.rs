//! Конфигурация мастер-сервера из переменных окружения.

mod s3;

use anyhow::{bail, Result};
pub use s3::S3Config;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Адрес для прослушивания, например "0.0.0.0:8080".
    pub bind_addr: String,
    /// Публичный базовый URL мастера ("https://api.example.dev"). Используется в
    /// URL файлов, Yggdrasil-манифесте и OAuth redirect.
    pub public_url: String,
    /// Публичный базовый URL сайта ("https://example.dev"): туда уходит игрок за
    /// экраном согласия OAuth2.
    pub web_url: String,
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

    /// Origin'ы, которым браузер разрешает ходить в API (через запятую в
    /// `NORO_ALLOWED_ORIGINS`). Пусто — CORS остаётся permissive: локальная
    /// разработка поднимает Nuxt на произвольном порту.
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // .env подхватывается вызывающим (main).
        Ok(Self {
            bind_addr: env_or("NORO_BIND", "0.0.0.0:8080"),
            public_url: env_required("NORO_PUBLIC_URL")?
                .trim_end_matches('/')
                .to_string(),
            web_url: env_required("NORO_WEB_URL")?
                .trim_end_matches('/')
                .to_string(),
            database_url: env_required("DATABASE_URL")?,
            data_dir: PathBuf::from(env_or("NORO_DATA_DIR", "./data")),
            discord_client_id: env_required("DISCORD_CLIENT_ID")?,
            discord_client_secret: env_required("DISCORD_CLIENT_SECRET")?,
            curseforge_api_key: env_opt("CURSEFORGE_API_KEY"),
            signing_key_hex: env_opt("NORO_SIGNING_KEY"),
            github_repo: env_opt("NORO_GITHUB_REPO"),
            github_token: env_opt("GITHUB_TOKEN"),
            github_ref: env_opt("NORO_GITHUB_REF"),
            launcher_repo_path: env_opt("NORO_LAUNCHER_REPO").map(PathBuf::from),
            files_cdn_url: env_opt("NORO_FILES_CDN_URL")
                .map(|u| u.trim_end_matches('/').to_string()),
            s3: S3Config::from_env()?,
            allowed_origins: env_or("NORO_ALLOWED_ORIGINS", "")
                .split(',')
                .map(|o| o.trim().trim_end_matches('/').to_string())
                .filter(|o| !o.is_empty())
                .collect(),
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

/// Обязательная переменная: без неё мастер не поднимается.
///
/// Дефолта тут быть не может. Раньше `NORO_PUBLIC_URL` молча становился
/// `http://localhost:8080`, и прод раздавал игрокам манифесты со ссылками на
/// localhost — ошибка всплывала у игрока, а не при старте.
fn env_required(key: &str) -> Result<String> {
    match env_opt(key) {
        Some(v) => Ok(v),
        None => bail!("переменная окружения {key} обязательна — см. .env.example"),
    }
}

/// Опциональная переменная. Пустая строка приравнена к отсутствию: в
/// docker-compose незаполненный `${VAR}` разворачивается именно в пустую.
fn env_opt(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn env_or(key: &str, default: &str) -> String {
    env_opt(key).unwrap_or_else(|| default.to_string())
}
