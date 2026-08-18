//! Что агент видит в профиле игрока.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct AgentRole {
    pub name: String,
    pub display_name: String,
    pub lp_group: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub sort_order: i32,
}

#[derive(Serialize)]
pub struct AgentPunishmentSummary {
    pub id: Uuid,
    pub kind: String,
    pub reason: String,
    pub actor_label: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rule_code: Option<String>,
}

impl From<crate::db::punishments::PunishmentRow> for AgentPunishmentSummary {
    fn from(row: crate::db::punishments::PunishmentRow) -> Self {
        Self {
            id: row.id,
            kind: row.kind,
            reason: row.reason,
            actor_label: row.actor_label,
            created_at: row.created_at,
            expires_at: row.expires_at,
            rule_code: row.rule_code,
        }
    }
}

#[derive(Serialize)]
pub struct AgentPlayer {
    pub uuid: Uuid,
    pub username: String,
    pub banned: bool,
    pub muted: bool,
    pub active_mute: Option<AgentPunishmentSummary>,
    pub active_ban: Option<AgentPunishmentSummary>,
    pub pending_warns: Vec<AgentPunishmentSummary>,
    pub allowed: bool,
    #[serde(default)]
    pub denial_reason: Option<String>,
    #[serde(default)]
    pub maintenance_bypass: bool,
    pub roles: Vec<AgentRole>,
    pub skin_url: String,
    pub cape_url: Option<String>,
    pub locale: Option<String>,
    pub lp_groups: Vec<String>,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub frozen: Option<schema::FreezeInfo>,
    #[serde(default)]
    pub vanish_on_join: bool,
}
