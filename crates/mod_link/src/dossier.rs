//! Player dossier: the question a moderator otherwise opens the site to answer,
//! which is whether this is a newcomer or their third ban for the same thing.

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
    /// First login. A two-day-old account reported for cheating reads
    /// differently from a two-year-old one.
    #[serde(default)]
    pub first_seen: Option<DateTime<Utc>>,
    /// Cases opened against this player, and how many of them were upheld.
    #[serde(default)]
    pub cases_total: i64,
    #[serde(default)]
    pub cases_confirmed: i64,
    /// Mutes, warnings and bans in force. Revoked and expired ones stay out —
    /// a hover wants the current state, the history is on the case card.
    #[serde(default)]
    pub active_punishments: Vec<CasePunishment>,
}
