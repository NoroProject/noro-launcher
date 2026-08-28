//! The contract between the launcher and the case-review client mod.
//!
//! The mod sends an intent (`ToLauncher`) and draws the state it is handed
//! (`ToMod`). There are no request ids and nothing correlates a reply to a call:
//! the mod is a subscriber, it doesn't ask and wait.
//!
//! Versions drift here in a way they never do inside the process — the mod
//! ships with a build and sits on people's machines for months while the
//! launcher updates itself. Hence `PROTOCOL`, `#[serde(default)]` on anything
//! new, and logging-and-skipping frames we don't recognise rather than failing.

mod case;
mod dossier;
mod player;
mod to_launcher;
mod to_mod;
#[cfg(test)]
mod wire_tests;

pub use case::{
    CaseBrief, CaseEvent, CaseMessage, CasePunishment, CaseReport, CaseView, InventorySlot,
};
pub use dossier::Dossier;
pub use player::{OwnPunishment, RuleCategory, RuleItem, RuleSanction};
pub use to_launcher::ToLauncher;
pub use to_mod::ToMod;

/// Bumped when an old mod can no longer understand a new launcher. A field with
/// a `default`, or a new frame, is not that.
pub const PROTOCOL: u32 = 1;

/// Cases per page in the queue.
///
/// The review panel lives in a corner of the screen and fits about ten rows.
/// Both sides know the number — the mod works out page offsets from it.
pub const QUEUE_PAGE: i64 = 10;

/// Handshake file, written into the instance directory rather than the
/// launcher's config: the mod only knows its own `gameDir` and shouldn't have to
/// guess where the launcher is installed.
pub const HANDSHAKE_FILE: &str = "noro-bridge.json";

/// The key is not decoration. The socket is reachable from outside the process
/// and any web page can open `ws://127.0.0.1:port` — CORS does not cover
/// WebSockets. What a page cannot do is read a file in the instance directory.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Handshake {
    pub port: u16,
    pub key: String,
    pub protocol: u32,
}
