//! Launcher → mod: state.
//!
//! The mod draws what it is sent and works nothing out for itself. Refusals
//! arrive as a translation key rather than finished text, because the interface
//! language lives in the mod's `lang` files.

use crate::case::{CaseBrief, CaseView, InventorySlot};
use crate::dossier::Dossier;
use crate::player::{OwnPunishment, RuleCategory, RuleItem, RuleSanction};
use crate::social::{ChatMessage, ChatThread, NewsPost, ProfileRole, Ticket, TicketMessage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ToMod {
    /// Handshake accepted. The permissions are here so the mod doesn't draw
    /// buttons that would be refused anyway — a convenience, not a check. The
    /// master is the one that decides.
    Ready {
        protocol: u32,
        username: String,
        /// Launcher language, `ru` or `en`.
        locale: String,
        permissions: Vec<String>,
    },
    /// A page of the queue. `total` counts everything matching the filter, so
    /// the panel can say "11-20 of 348" and knows whether more exists.
    ///
    /// `query` and `offset` echo the request, so the panel can tell a page it
    /// asked for from an unsolicited queue update. Without them it would
    /// announce a new case every time someone turned a page.
    Queue {
        cases: Vec<CaseBrief>,
        total: i64,
        offset: i64,
        #[serde(default)]
        query: Option<String>,
    },
    /// Resource packs were swapped under the running game. The client still has
    /// the old ones in memory, so it needs a resource reload — and only the mod
    /// can ask for one, there is no such handle outside the game process.
    ReloadResources {
        packs: Vec<String>,
    },
    /// The whole card, chat slice included; the slice is part of the card here
    /// exactly as it is on the site.
    ///
    /// Boxed because the card is an order of magnitude larger than any other
    /// frame, and a two-line `Notice` would otherwise cost the same.
    Case {
        view: Box<CaseView>,
    },
    Dossier {
        dossier: Dossier,
    },
    /// The inventory snapshot gets its own frame even though the same data sits
    /// in the timeline: there it's raw JSON, and parsing that in Java is work
    /// for nothing.
    Inventory {
        case_id: Uuid,
        items: Vec<InventorySlot>,
    },
    /// Refused. `intent` names the frame that was refused so the mod knows which
    /// button to give up on; `reason` is a translation key.
    Rejected {
        intent: String,
        reason: String,
        /// Code from the master's registry. `0` means the refusal came from
        /// somewhere else — no network, or a local check. The panel shows it
        /// next to the text: the reason is translated locally, but the number is
        /// the same everywhere and can be quoted.
        #[serde(default)]
        number: u16,
    },
    /// Sections, rules and sanction ranges in one frame rather than three. The
    /// rules get opened to read a clause together with what it costs, and
    /// loading the ranges separately would show the rule without the
    /// consequence.
    Rules {
        categories: Vec<RuleCategory>,
        rules: Vec<RuleItem>,
        sanctions: Vec<RuleSanction>,
    },
    /// The player's whole history, revoked entries included.
    OwnPunishments {
        punishments: Vec<OwnPunishment>,
    },
    /// Who is playing. Sent right after `Ready` — the launcher already holds the
    /// profile, so this costs the master nothing.
    Profile {
        username: String,
        uuid: Uuid,
        #[serde(default)]
        roles: Vec<ProfileRole>,
        #[serde(default)]
        joined_at: Option<DateTime<Utc>>,
        /// Joins servers already vanished. Only staff with a vanish permission
        /// can change it; the mod shows the switch to them alone.
        #[serde(default)]
        silent_join: bool,
    },
    /// The news channel, newest first. Re-sent whenever the master says the
    /// news changed.
    News {
        posts: Vec<NewsPost>,
    },
    /// A picture for a news post, re-encoded as a small PNG — the game reads
    /// nothing else. `None` means it couldn't be fetched, so the mod stops
    /// waiting for it.
    Image {
        url: String,
        #[serde(default)]
        png_base64: Option<String>,
    },
    /// The player's conversations, freshest first.
    Chats {
        threads: Vec<ChatThread>,
        total: i64,
        /// Unread messages across all of them, for the dock badge.
        #[serde(default)]
        unread: i64,
    },
    /// One conversation, oldest message first. Re-sent while it's open whenever
    /// a message arrives in it.
    Chat {
        peer_id: Uuid,
        peer_name: String,
        messages: Vec<ChatMessage>,
        total: i64,
    },
    Tickets {
        tickets: Vec<Ticket>,
        total: i64,
    },
    /// One ticket's messages, oldest first.
    TicketMessages {
        ticket_id: Uuid,
        messages: Vec<TicketMessage>,
        total: i64,
    },
    /// Something to say in words, by key, like the master's `Notification`.
    Notice {
        key: String,
        #[serde(default)]
        args: BTreeMap<String, String>,
    },
}
