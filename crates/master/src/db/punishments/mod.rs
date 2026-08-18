//! Наказания: баны со сроком, предупреждения, ограничения по серверу.
//!
//! Разделено на чтение и запись, а не на «баны» и «муты»: вид наказания — это
//! колонка `kind`, а вот выборки и правки живут по разным правилам. Выборки
//! спрашивают с игрового сервера на каждом входе, правки идут из админки.

pub mod read;
pub mod write;

pub use read::*;
pub use write::*;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PunishmentRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: String,
    pub reason: String,
    pub actor_id: Option<Uuid>,
    pub actor_label: String,
    pub server_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    /// `None` — навсегда.
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// Правило, по которому выдано. `None` — правило не указывали.
    pub rule_id: Option<Uuid>,
    /// Код правила на момент выдачи: правило переименуют, а разбор через год
    /// должен показывать, за что наказали.
    pub rule_code: Option<String>,
}

impl PunishmentRow {
    /// Действует ли прямо сейчас.
    pub fn active(&self) -> bool {
        self.revoked_at.is_none() && self.expires_at.is_none_or(|e| e > Utc::now())
    }
}

/// Что записываем в журнал наказаний. Структурой, а не девятью аргументами:
/// перепутать местами два `Option<Uuid>` в вызове было слишком легко.
#[derive(Debug, Clone)]
pub struct NewPunishment<'a> {
    pub user_id: Uuid,
    pub kind: &'a str,
    pub reason: &'a str,
    pub actor_id: Option<Uuid>,
    pub actor_label: &'a str,
    pub server_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rule_id: Option<Uuid>,
    pub rule_code: Option<&'a str>,
}
