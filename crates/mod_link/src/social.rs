//! What the player's side panel shows: the profile, the news channel, direct
//! messages and support tickets.
//!
//! Every shape here is the launcher's own, not the master's. The master's
//! answers carry presence, sources and paging the panel never draws, and
//! passing them through would tie a jar that already shipped to whatever the
//! site needs this month.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A role as the profile card draws it: a name and its colour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRole {
    pub name: String,
    /// `#RRGGBB`, or nothing for a role without a colour.
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsPost {
    pub id: Uuid,
    pub title: String,
    /// Markdown, images included. Image links are absolute by the time they get
    /// here: the mod never learns the master's address.
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub author_name: Option<String>,
    #[serde(default)]
    pub pinned: bool,
    pub published_at: DateTime<Utc>,
}

/// One conversation in the list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatThread {
    pub peer_id: Uuid,
    pub peer_name: String,
    #[serde(default)]
    pub peer_is_bot: bool,
    #[serde(default)]
    pub preview: String,
    #[serde(default)]
    pub preview_is_mine: bool,
    #[serde(default)]
    pub unread: i64,
    pub last_message_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub author_name: String,
    pub body: String,
    pub at: DateTime<Utc>,
    /// Written by whoever is playing — the mod draws these on the other side.
    #[serde(default)]
    pub mine: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub subject: String,
    /// `open`, `answered`, `closed`.
    pub status: String,
    #[serde(default)]
    pub unread: i64,
    pub last_message_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMessage {
    pub id: Uuid,
    /// `player` or `staff`; the mod lines staff replies up on the left.
    pub author_side: String,
    pub author_name: String,
    #[serde(default)]
    pub author_role: Option<String>,
    #[serde(default)]
    pub author_role_color: Option<String>,
    pub content: String,
    pub at: DateTime<Utc>,
}
