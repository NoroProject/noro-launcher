//! What an ordinary player gets: the rules, and their own punishments.
//!
//! Neither needs moderator permissions or a new endpoint on the master. The
//! rules are public on purpose — every ban cites one, and the banned player has
//! to be able to read which — and their own punishments are already on their
//! account page. The mod just puts both where the question comes up.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// A section of the rules. Items without a section are shown last.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCategory {
    pub id: Uuid,
    /// The master calls this `name`. It's an alias, not a rename: the mod
    /// receives `title` here as well as on a rule item, so the two aren't
    /// spelled differently for the same thing.
    #[serde(alias = "name", default)]
    pub title: String,
    #[serde(default)]
    pub sort_order: i32,
}

/// The sanction range for a rule: what, and for how long.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSanction {
    pub rule_id: Uuid,
    pub kind: String,
    #[serde(default)]
    pub min_minutes: Option<i64>,
    #[serde(default)]
    pub max_minutes: Option<i64>,
}

/// A punishment of the player asking, never of anyone else.
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
    /// Revoked and expired punishments stay in the list, they just aren't
    /// active: a lifted ban still happened, and it matters at the next case.
    pub fn active(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at.is_none_or(|at| at > now)
    }
}
