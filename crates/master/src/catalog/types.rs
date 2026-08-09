//! Нормализованные типы каталога.
//!
//! Modrinth и CurseForge отдают разное по форме одно и то же. Приводим к общему
//! виду здесь, один раз, — иначе развилка «какой провайдер» расползётся по всем
//! компонентам админки.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Modrinth,
    Curseforge,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Modrinth => "modrinth",
            Provider::Curseforge => "curseforge",
        }
    }
}

/// Карточка в выдаче поиска.
#[derive(Serialize, Clone)]
pub struct ModHit {
    pub provider: Provider,
    /// Строкой у обоих: у Modrinth это base62, у CurseForge — число.
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub follows: u64,
    pub categories: Vec<String>,
    /// `required` | `optional` | `unsupported` | `unknown`. CurseForge про
    /// стороны ничего не знает, поэтому там всегда `unknown`.
    pub client_side: String,
    pub server_side: String,
    pub updated: Option<String>,
    pub page_url: Option<String>,
}

/// Карточка проекта целиком: то, что показываем в панели справа.
#[derive(Serialize, Clone)]
pub struct ModProject {
    #[serde(flatten)]
    pub hit: ModHit,
    /// Markdown у Modrinth, готовый HTML у CurseForge — веб знает по провайдеру.
    pub body: String,
    pub gallery: Vec<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub license: Option<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct ModDependency {
    pub project_id: Option<String>,
    /// `required` | `optional` | `incompatible` | `embedded`.
    pub kind: String,
}

#[derive(Serialize, Clone)]
pub struct ModVersion {
    pub provider: Provider,
    /// Для установки: version_id у Modrinth, file_id у CurseForge.
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    /// `release` | `beta` | `alpha`.
    pub channel: String,
    pub downloads: u64,
    pub published: Option<String>,
    pub filename: String,
    pub size: u64,
    pub dependencies: Vec<ModDependency>,
    /// CurseForge разрешает автору запретить стороннюю загрузку. Такую версию
    /// показываем, но ставить не даём — честнее, чем прятать.
    pub downloadable: bool,
}

#[derive(Serialize, Clone)]
pub struct Category {
    pub name: String,
    pub display: String,
    pub icon: Option<String>,
}

/// Откуда берём файл. Общий вход и для сборки, и для игрового сервера.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModSource {
    Modrinth {
        version_id: String,
    },
    Curseforge {
        project_id: u64,
        file_id: u64,
    },
    Url {
        url: String,
        #[serde(default)]
        filename: Option<String>,
    },
}

/// Разобранный источник: файл уже в сторе, метаданные — для карточки
/// опционального мода.
pub struct ResolvedMod {
    pub filename: String,
    pub sha1: String,
    pub size: u64,
    pub title: Option<String>,
    pub author: Option<String>,
    pub icon_url: Option<String>,
}
