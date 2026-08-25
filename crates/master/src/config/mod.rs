//! Конфигурация мастер-сервера.
//!
//! Часть значений живёт в БД (`instance_settings`), часть — только в env.
//! Приоритет чтения: **env > БД > дефолт**.
//!
//! Читается один раз при старте и дальше не меняется. Применение новых настроек
//! — рестарт мастера: `Config` торчит в мастере повсюду, `ArcSwap` задел бы
//! много файлов, а часть полей всё равно требует рестарта. Цена рестарта —
//! оборванные WS-лаунчеров (переподключатся сами) и прерванные загрузки, потому
//! админка после сохранения показывает «требуется перезапуск» и не делает его
//! сама: момент выбирает оператор.

pub mod keys;
mod load;
mod s3;

pub use s3::S3Config;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Адрес для прослушивания, например "0.0.0.0:8080". Только env: до подъёма
    /// БД мастер уже должен знать, где слушать.
    pub bind_addr: String,
    /// Строка подключения к PostgreSQL. Только env по той же причине.
    pub database_url: String,
    /// Корень данных: FileStore, ключи, временные файлы. Только env.
    pub data_dir: PathBuf,

    /// Публичный базовый URL мастера ("https://api.example.dev"). Пусто до
    /// завершения первичной настройки.
    pub public_url: String,
    /// Публичный базовый URL сайта ("https://example.dev").
    pub web_url: String,
    /// Имя инстанса — для заголовков и писем.
    pub instance_name: String,

    pub curseforge_api_key: Option<String>,

    /// ed25519 приватный ключ (hex, 32 байта seed). Если пусто — dev-режим.
    pub signing_key_hex: Option<String>,

    pub github_repo: Option<String>,
    pub github_token: Option<String>,
    pub github_ref: Option<String>,
    pub launcher_repo_path: Option<PathBuf>,

    /// Опциональный публичный URL для отдачи файлов (CDN, S3, R2, MinIO).
    pub files_cdn_url: Option<String>,

    pub s3: Option<S3Config>,

    /// Origin'ы, которым браузер разрешает ходить в API. Пусто — CORS остаётся
    /// permissive: локальная разработка поднимает Nuxt на произвольном порту.
    pub allowed_origins: Vec<String>,
}

impl Config {
    /// URL для скачивания файла по SHA1.
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

    pub fn is_dev_signing(&self) -> bool {
        self.signing_key_hex.is_none()
    }

    /// Настроен ли инстанс достаточно, чтобы обслуживать игроков.
    ///
    /// Без публичных адресов манифесты уезжали бы со ссылками в никуда, а
    /// OAuth-редирект вёл бы на пустую строку.
    pub fn is_usable(&self) -> bool {
        !self.public_url.is_empty() && !self.web_url.is_empty()
    }
}

pub use load::{env_opt, env_or, env_required, Bootstrap};

#[cfg(test)]
impl Config {
    /// Конфиг для тестов: заполнено только то, без чего проверяемая функция не
    /// работает. Раньше тесты выставляли переменные окружения и собирали конфиг
    /// целиком — то есть зависели от порядка запуска.
    pub fn for_test(public_url: &str, files_cdn_url: Option<&str>) -> Self {
        Self {
            bind_addr: "127.0.0.1:0".into(),
            database_url: "postgres://localhost/noro_test".into(),
            data_dir: PathBuf::from("./data"),
            public_url: public_url.trim_end_matches('/').to_string(),
            web_url: "https://example.dev".into(),
            instance_name: "Noro Test".into(),
            curseforge_api_key: None,
            signing_key_hex: None,
            github_repo: None,
            github_token: None,
            github_ref: None,
            launcher_repo_path: None,
            files_cdn_url: files_cdn_url.map(|u| u.trim_end_matches('/').to_string()),
            s3: None,
            allowed_origins: Vec::new(),
        }
    }
}
