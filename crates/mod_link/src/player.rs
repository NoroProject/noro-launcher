//! Что нужно обычному игроку: свод правил и свои наказания.
//!
//! Ни то ни другое не требует прав модератора и не заводит новых ручек на
//! мастере: свод публичен намеренно — на него ссылается каждый бан, и забаненный
//! обязан прочитать, за что именно, — а свои наказания игрок и так видит в
//! кабинете. Мод переносит это в игру, где вопрос и возникает.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Пункт свода.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleItem {
    pub id: Uuid,
    #[serde(default)]
    pub category_id: Option<Uuid>,
    pub code: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sort_order: i32,
}

/// Раздел свода: пункты без раздела показываются в конце.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCategory {
    pub id: Uuid,
    /// У мастера это поле зовётся `name`. Псевдоним, а не переименование:
    /// моду уезжает `title` — то же слово, что у пункта свода, чтобы у раздела
    /// и пункта не было двух разных имён для одного и того же.
    #[serde(alias = "name", default)]
    pub title: String,
    #[serde(default)]
    pub sort_order: i32,
}

/// Вилка наказания по пункту: за что и насколько.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSanction {
    pub rule_id: Uuid,
    pub kind: String,
    #[serde(default)]
    pub min_minutes: Option<i64>,
    #[serde(default)]
    pub max_minutes: Option<i64>,
}

/// Наказание игрока — своё, а не чужое.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnPunishment {
    pub id: Uuid,
    pub kind: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub revoked_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub rule_code: Option<String>,
}

impl OwnPunishment {
    /// Действует ли прямо сейчас. Снятое и истёкшее остаётся в истории: снятый
    /// бан — тоже факт, и он нужен при разборе следующего случая.
    pub fn active(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at.is_none_or(|at| at > now)
    }
}
