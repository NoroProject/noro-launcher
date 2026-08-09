//! Параметры поиска по каталогу — общие для обоих провайдеров.

use super::types::ModHit;
use serde::{Deserialize, Serialize};

fn default_type() -> String {
    "mod".into()
}
fn default_sort() -> String {
    "relevance".into()
}
fn default_limit() -> u32 {
    20
}
fn default_provider() -> String {
    "modrinth".into()
}

#[derive(Deserialize, Clone)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    pub mc: Option<String>,
    pub loader: Option<String>,
    /// Через запятую: `magic,technology`. Повторяющиеся ключи в query-строке
    /// axum разобрать в `Vec` не умеет, а городить ради этого свой extractor —
    /// больше кода, чем один `split`.
    #[serde(default)]
    pub categories: String,
    #[serde(default = "default_type")]
    pub project_type: String,
    /// `relevance` | `downloads` | `follows` | `newest` | `updated`.
    #[serde(default = "default_sort")]
    pub sort: String,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// `modrinth` | `curseforge` | `all`.
    #[serde(default = "default_provider")]
    pub provider: String,
    /// `client` | `server`.
    pub side: Option<String>,
}

impl SearchQuery {
    pub fn category_list(&self) -> Vec<&str> {
        self.categories
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Потолок страницы: 100 — предел у обоих API, а без ограничения сюда
    /// прилетит `limit=100000` и мы честно его переспросим у Modrinth.
    pub fn page_size(&self) -> u32 {
        self.limit.clamp(1, 100)
    }
}

#[derive(Serialize)]
pub struct SearchPage {
    pub hits: Vec<ModHit>,
    pub total: u64,
    /// Провайдеры, которые не ответили. Выдачу второго это не отменяет — веб
    /// покажет предупреждение и то, что нашлось.
    pub failed: Vec<String>,
}
