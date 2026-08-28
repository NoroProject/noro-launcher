//! Карточка дела в том виде, в каком её показывает мод.
//!
//! Формы повторяют то, что отдаёт админ-API мастера, — но это договор
//! лаунчера с модом, а не сквозной проброс: разбирает ответ мастера лаунчер,
//! и он же обновляется вместе с ним. Переименование поля на мастере ломается
//! здесь, при разборе, а не в чужом jar, который уже у людей.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Строка очереди: столько, сколько нужно, чтобы выбрать следующее дело.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseBrief {
    pub id: Uuid,
    /// Человеческий номер: печатается как `N-000000001`.
    pub number: i64,
    pub target_id: Uuid,
    pub target_name: Option<String>,
    pub game_server_id: Option<Uuid>,
    pub server_name: Option<String>,
    pub status: String,
    pub claimed_by: Option<Uuid>,
    pub claimed_by_name: Option<String>,
    pub opened_at: DateTime<Utc>,
    #[serde(default)]
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub verdict: Option<String>,
    #[serde(default)]
    pub rule_code: Option<String>,
    /// Жалоб и сколько *разных* людей их написали: десять от одного обиженного
    /// и от десяти разных — разный вес.
    #[serde(default)]
    pub reports_count: i64,
    #[serde(default)]
    pub reporters_count: i64,
    #[serde(default)]
    pub last_report_at: Option<DateTime<Utc>>,
}

/// Лента разбора. `payload` остаётся сырым JSON: у двух десятков видов событий
/// нет общей формы, а превращать её в плоскую запись со всеми полями всех
/// событий — ровно та ошибка, которой стоило избежать в `LinkFrame`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseEvent {
    pub id: Uuid,
    pub at: DateTime<Utc>,
    #[serde(default)]
    pub actor_label: String,
    /// `web`, `game` или `system`.
    pub source: String,
    pub kind: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseMessage {
    pub id: Uuid,
    pub at: DateTime<Utc>,
    pub sender_name: String,
    /// `public`, `local`, `private` или `command`.
    pub channel: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseReport {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub reporter_name: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub world: Option<String>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub z: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasePunishment {
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

/// Сколько жалоб человека подтвердилось — вес его слова в очереди.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReporterStats {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub confirmed: i64,
    #[serde(default)]
    pub rejected: i64,
}

/// Карточка целиком — одним кадром, как и одним запросом у сайта: четыре
/// панели, догружающиеся по очереди, читаются как «лаунчер подвис».
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseView {
    #[serde(rename = "case")]
    pub brief: CaseBrief,
    #[serde(default)]
    pub reports: Vec<CaseReport>,
    #[serde(default)]
    pub events: Vec<CaseEvent>,
    #[serde(default)]
    pub punishments: Vec<CasePunishment>,
    /// Срез чата. Пуст, когда нет права `noro.mod.cases.chat`, — и тогда
    /// `chat_allowed` говорит, что это отказ, а не пустой буфер.
    #[serde(default)]
    pub messages: Vec<CaseMessage>,
    #[serde(default)]
    pub chat_allowed: bool,
    /// Репутация жалобщиков, ключ — `reporter_id` строкой.
    #[serde(default)]
    pub reporters: BTreeMap<String, ReporterStats>,
}

/// Занятый слот в снимке инвентаря.
///
/// `nbt` — предмет целиком в JSON, каким его отдал сервер: по нему панель
/// рисует настоящую иконку с зачарованиями и переименованием. Пусто — платформа
/// так не умеет, и остаётся показать снимок текстом.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    #[serde(default)]
    pub slot: i32,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub count: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub nbt: Option<String>,
}
