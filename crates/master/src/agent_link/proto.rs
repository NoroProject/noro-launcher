//! Frames on the master ↔ agent channel.
//!
//! Down (`ToAgent`) is "here's what changed, apply it now". Punishments are
//! still issued over plain HTTP, because those need a response body. Up
//! (`FromAgent`) is game events the master has no other way to see.
//!
//! Players are addressed by MC UUID rather than `users.id`: that's the only
//! identity the agent has, and mapping it on the game side would mean a second
//! copy of the database there.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A punishment as it is shown to the player.
#[derive(Serialize, Clone, Debug)]
pub struct LivePunishment {
    pub id: Uuid,
    /// MC UUID of the punished player.
    pub target: Uuid,
    pub target_name: String,
    /// `ban` | `server_ban` | `mute` | `warn`.
    pub kind: String,
    pub reason: String,
    pub actor_label: String,
    pub created_at: DateTime<Utc>,
    /// `None` means permanent.
    pub expires_at: Option<DateTime<Utc>>,
    pub rule_code: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToAgent {
    /// Punishment issued or extended — apply immediately.
    Punished {
        punishment: LivePunishment,
    },
    /// Punishment lifted. `kind` tells the agent what to release: a lifted mute
    /// gives chat back, a lifted ban does nothing to anyone online.
    Revoked {
        id: Uuid,
        target: Uuid,
        target_name: String,
        kind: String,
        actor_label: String,
    },
    /// Message or automod settings changed in the admin panel — re-read them.
    MessagesChanged,
    FiltersChanged,
    RestartNotice {
        seconds: u32,
        reason: Option<String>,
    },
    /// A player's roles, permissions or prefixes changed — re-read the profile.
    ///
    /// `None` means "re-read everyone", which is how edits to a role itself
    /// travel. The agent batches those; on a server with a hundred players it
    /// would otherwise be a hundred requests back to back.
    ProfileChanged {
        uuid: Option<Uuid>,
    },
    Kick {
        target: Uuid,
        message: String,
    },
    /// Private message to one player.
    Tell {
        target: Uuid,
        message: String,
    },
    Announce {
        message: String,
    },
    /// Maintenance started: warn, then kick everyone without bypass after
    /// `countdown_seconds`.
    MaintenanceStart {
        countdown_seconds: u32,
        reason: Option<String>,
    },
    MaintenanceCancel,
    /// A case was claimed — put the moderator into review mode if they're online.
    ///
    /// Everything the menu needs rides along in the frame; fetching the card
    /// from the master would delay the menu exactly when the server is busy.
    CaseAssigned {
        case: Uuid,
        target: Uuid,
        target_name: String,
        moderator: Uuid,
        reason: String,
        world: Option<String>,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        reporter: Option<Uuid>,
        reporter_name: Option<String>,
    },
    /// Case released or closed — leave review mode, put the moderator back.
    CaseFinished {
        case: Uuid,
        moderator: Uuid,
        closed: bool,
    },
    /// Send chat around the event. `before_secs` is how far back to rewind in
    /// the agent's buffer.
    CaseChatRequest {
        case: Uuid,
        target: Uuid,
        before_secs: u32,
    },
    CaseInventoryRequest {
        case: Uuid,
        target: Uuid,
    },
}

/// Agent → master frames.
///
/// An event is not the source of truth. The channel drops and frames get lost,
/// so nothing irreversible happens off a single frame: the online roster is
/// reconciled by heartbeat, and events only exist to react quickly. Unknown
/// frames are ignored silently — an old master has to survive a newer agent.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FromAgent {
    /// `ip_hash` is already hashed; the master has no use for the raw address.
    PlayerJoin {
        uuid: Uuid,
        #[serde(default)]
        ip_hash: Option<String>,
        /// Vanished — must not appear in the public online list.
        #[serde(default)]
        vanished: bool,
    },
    /// `reason` is free text for the log.
    PlayerLeave {
        uuid: Uuid,
        #[serde(default)]
        reason: Option<String>,
    },
    /// The game thread didn't move for `stalled_secs` seconds.
    TickStall { stalled_secs: u32 },
    /// Moderator claimed a case in-game. The master takes the lock: an in-game
    /// command and a button on the site can equally lose the race.
    CaseClaim { case: Uuid, moderator: Uuid },
    /// What the moderator did in review mode: teleport, freeze, spectate.
    /// `payload` goes into the feed as-is — whoever renders it knows the shape.
    CaseAction {
        case: Uuid,
        moderator: Uuid,
        kind: String,
        #[serde(default)]
        payload: serde_json::Value,
    },
    CaseChatSlice {
        case: Uuid,
        messages: Vec<crate::db::cases::IncomingMessage>,
    },
    /// Inventory snapshot: dupe evidence belongs in the case, not in memory.
    CaseInventory {
        case: Uuid,
        moderator: Option<Uuid>,
        items: serde_json::Value,
    },
}
