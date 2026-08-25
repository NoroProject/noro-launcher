//! Сборка `Config` из окружения и БД.

use super::keys::{self, Key};
use super::{Config, S3Config};
use anyhow::{bail, Result};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// То, что нужно знать до подъёма БД.
#[derive(Debug, Clone)]
pub struct Bootstrap {
    pub bind_addr: String,
    pub database_url: String,
    pub data_dir: PathBuf,
}

impl Bootstrap {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind_addr: env_or("NORO_BIND", "0.0.0.0:8080"),
            database_url: env_required("DATABASE_URL")?,
            data_dir: PathBuf::from(env_or("NORO_DATA_DIR", "./data")),
        })
    }
}

impl Config {
    /// Собрать конфигурацию: env перекрывает БД, БД перекрывает дефолт.
    pub async fn load(boot: Bootstrap, pool: &PgPool) -> Result<Self> {
        let stored = crate::db::all_settings(pool).await?;
        let get = |k: Key| setting(&stored, k);

        Ok(Self {
            bind_addr: boot.bind_addr,
            database_url: boot.database_url,
            data_dir: boot.data_dir,

            public_url: get(keys::PUBLIC_URL)
                .map(|v| v.trim_end_matches('/').to_string())
                .unwrap_or_default(),
            web_url: get(keys::WEB_URL)
                .map(|v| v.trim_end_matches('/').to_string())
                .unwrap_or_default(),
            instance_name: get(keys::INSTANCE_NAME).unwrap_or_else(|| "Noro Network".into()),

            curseforge_api_key: env_opt("CURSEFORGE_API_KEY"),
            signing_key_hex: env_opt("NORO_SIGNING_KEY"),

            github_repo: get(keys::GITHUB_REPO),
            github_token: env_opt("GITHUB_TOKEN"),
            github_ref: get(keys::GITHUB_REF),
            launcher_repo_path: get(keys::LAUNCHER_REPO).map(PathBuf::from),

            files_cdn_url: get(keys::FILES_CDN_URL).map(|u| u.trim_end_matches('/').to_string()),
            s3: S3Config::from_env()?,

            allowed_origins: get(keys::ALLOWED_ORIGINS)
                .unwrap_or_default()
                .split(',')
                .map(|o| o.trim().trim_end_matches('/').to_string())
                .filter(|o| !o.is_empty())
                .collect(),
        })
    }
}

/// Значение настройки: сначала env, потом БД. Пустая строка приравнена к
/// отсутствию — в docker-compose незаполненный `${VAR}` разворачивается именно
/// в неё.
fn setting(stored: &BTreeMap<String, Value>, key: Key) -> Option<String> {
    if let Some(v) = env_opt(key.env) {
        return Some(v);
    }
    stored
        .get(key.name)
        .and_then(as_text)
        .filter(|v| !v.is_empty())
}

/// В JSONB настройки лежат строками, но руками туда легко положить число или
/// булев — читаем и такое, вместо того чтобы молча вернуть «не задано».
fn as_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Null => None,
        other => Some(other.to_string()),
    }
}

/// Обязательная переменная: без неё мастер не поднимается.
pub fn env_required(key: &str) -> Result<String> {
    match env_opt(key) {
        Some(v) => Ok(v),
        None => bail!("переменная окружения {key} обязательна — см. .env.example"),
    }
}

/// Опциональная переменная. Пустая строка приравнена к отсутствию.
pub fn env_opt(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub fn env_or(key: &str, default: &str) -> String {
    env_opt(key).unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
#[path = "load_tests.rs"]
mod tests;
