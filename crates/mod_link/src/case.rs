//! A case as the mod shows it.
//!
//! The shapes mirror the master's admin API, but this is the launcher's contract
//! with the mod, not a pass-through. The launcher parses the master's response,
//! so a renamed field breaks here, where it can be fixed with an update, instead
//! of inside a jar that is already on people's machines.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// One row of the queue — enough to choose what to look at next.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseBrief {
    pub id: Uuid,
    /// The number people quote, printed as `N-000000001`.
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
    /// Reports, and how many *different* people filed them. Ten from one
    /// aggrieved player is not ten from ten.
    #[serde(default)]
    pub reports_count: i64,
    #[serde(default)]
    pub reporters_count: i64,
    #[serde(default)]
    pub last_report_at: Option<DateTime<Utc>>,
}

/// The case timeline. `payload` stays raw JSON — two dozen kinds of event have
/// no shape in common, and flattening them into one record with every field of
/// every event helps nobody.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseEvent {
    pub id: Uuid,
    pub at: DateTime<Utc>,
    #[serde(default)]
    pub actor_label: String,
    /// `web`, `game` or `system`.
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
    /// `public`, `local`, `private` or `command`.
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

/// How many of this player's reports held up, i.e. what their word is worth.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReporterStats {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub confirmed: i64,
    #[serde(default)]
    pub rejected: i64,
}

/// The whole card in a single frame, the way the site fetches it in a single
/// request. Four panels filling in one after another reads as a stuck launcher.
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
    /// Chat around the incident. Empty without `noro.mod.cases.chat`, and then
    /// `chat_allowed` is what tells the panel it was refused rather than quiet.
    #[serde(default)]
    pub messages: Vec<CaseMessage>,
    #[serde(default)]
    pub chat_allowed: bool,
    /// Keyed by `reporter_id` as a string.
    #[serde(default)]
    pub reporters: BTreeMap<String, ReporterStats>,
}

/// One occupied slot of an inventory snapshot.
///
/// `nbt` is the whole item as JSON, exactly as the server sent it — the panel
/// draws a real icon from it, enchantments and rename included. Empty when the
/// platform can't produce it, and then the snapshot is text only.
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
