//! Mod → launcher: intents.
//!
//! The mod says what it wants and never learns which request carries it. No
//! admin URL lives in the mod and none ever will — moving an endpoint would
//! break a jar that already shipped.
//!
//! Nothing here acts on the world. Teleport, freeze and spectate go out as the
//! same `/case …` commands the chat buttons send, so adding a panel doesn't mean
//! rebuilding the server agent.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ToLauncher {
    /// First frame of the connection; nothing else is accepted before it.
    Hello {
        key: String,
        protocol: u32,
    },
    /// The whole queue from the top, unfiltered. Kept for mods that shipped with
    /// older builds and know no other frame.
    ///
    /// Fields must never be added to it: a frame without fields has no `data`,
    /// so `{"type": "RequestQueue"}` from an old mod would stop parsing
    /// entirely. Newer mods send `RequestQueuePage`.
    RequestQueue,
    /// The master searches and paginates. A filter that lived in the mod could
    /// only see the page it had already loaded, so "no such case" really meant
    /// "not on the first page".
    RequestQueuePage {
        /// Nickname, server, moderator or case number. Empty means everything.
        #[serde(default)]
        query: Option<String>,
        #[serde(default)]
        offset: i64,
    },
    /// The launcher remembers the case as open and re-sends it on every
    /// `CaseUpdated` from the master until `CloseCase`.
    OpenCase {
        case_id: Uuid,
    },
    CloseCase,
    Claim {
        case_id: Uuid,
    },
    Release {
        case_id: Uuid,
    },
    Resolve {
        case_id: Uuid,
        /// `confirmed`, `rejected` or `insufficient`.
        verdict: String,
        #[serde(default)]
        resolution: String,
        #[serde(default)]
        rule_code: Option<String>,
    },
    AddNote {
        case_id: Uuid,
        text: String,
    },
    Punish {
        case_id: Uuid,
        /// `mute`, `warn`, `kick`, `ban`.
        kind: String,
        reason: String,
        #[serde(default)]
        rule_code: Option<String>,
        /// Empty means permanent, same as in the admin panel.
        #[serde(default)]
        duration_secs: Option<i64>,
    },
    /// Ask the server for the chat around the incident. The answer doesn't come
    /// back as a reply — the agent takes the slice, which takes time, and it
    /// arrives as an updated card.
    RequestChat {
        case_id: Uuid,
    },
    RequestInventory {
        case_id: Uuid,
    },
    /// A screenshot for the case. The PNG travels base64 — the channel is text,
    /// and a binary frame for this one case would complicate both sides.
    Attach {
        case_id: Uuid,
        #[serde(default)]
        note: String,
        png_base64: String,
    },
    /// A pointer to a chat message, not the message itself.
    ///
    /// The client is not a source of evidence: what goes into the case is the
    /// line from the agent's `ChatRing`, found by sender and time. The hash only
    /// confirms both sides mean the same line.
    Quote {
        case_id: Uuid,
        sender: String,
        at: DateTime<Utc>,
        hash: String,
    },
    Lookup {
        username: String,
    },
    /// A public document — no permission required.
    RequestRules,
    RequestOwnPunishments,
}
