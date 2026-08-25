//! Досье игрока: тот же вопрос, ради которого сейчас уходят на сайт, —
//! «он новичок или у него третий бан за то же».

use crate::case::CasePunishment;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dossier {
    pub user_id: Uuid,
    pub username: String,
    #[serde(default)]
    pub roles: Vec<String>,
    /// Дата первого входа. Двухдневный аккаунт с жалобой на чит читается иначе,
    /// чем двухлетний.
    #[serde(default)]
    pub first_seen: Option<DateTime<Utc>>,
    /// Дел на этого игрока и сколько из них подтвердилось.
    #[serde(default)]
    pub cases_total: i64,
    #[serde(default)]
    pub cases_confirmed: i64,
    /// Действующие муты, варны и баны. Снятые и истёкшие сюда не попадают:
    /// в наведении важно текущее состояние, а история — в карточке дела.
    #[serde(default)]
    pub active_punishments: Vec<CasePunishment>,
}
