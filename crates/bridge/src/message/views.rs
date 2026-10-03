// Over 150 lines: plain data, one struct for each kind of thing the window
// lists.
//! What the window shows, as plain data: catalogue hits, punishments, rules,
//! tickets, conversations.

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSettingsState {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
    #[serde(default)]
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogHitInfo {
    pub provider: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub author: Option<String>,
    pub downloads: u64,
}

/// The full mod page, fetched on demand: search results carry neither the
/// description nor the screenshots, and pulling them for every card in a list
/// would be pointless.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModProjectInfo {
    pub provider: String,
    pub project_id: String,
    /// Markdown from Modrinth, HTML from CurseForge.
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub gallery: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub page_url: Option<String>,
    pub license: Option<String>,
}

/// One version of a catalogue project, as the version picker shows it.
///
/// A picker and not "install the newest": the newest build of a mod is
/// routinely published for a Minecraft version the build is not on yet, and
/// installing it silently is how a launcher earns a reputation for breaking
/// games.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentVersionInfo {
    pub id: String,
    pub name: String,
    pub version_number: String,
    /// `release` · `beta` · `alpha`.
    pub channel: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u64,
    pub filename: String,
    pub size: u64,
    /// CurseForge lets an author forbid third-party downloads. Shown, not
    /// hidden — "why is this one missing" is a worse question than a greyed row.
    pub downloadable: bool,
    /// Fits the build this picker was opened for.
    pub compatible: bool,
}

/// A punishment on this account, as the player's own page shows it.
///
/// Trimmed down from what the master stores: staff ids and revocation
/// bookkeeping are not something the punished person needs, and "who banned
/// you" is already a name in `actor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunishmentView {
    pub kind: String,
    pub reason: String,
    pub actor: String,
    /// Unix seconds. Formatting is the frontend's business.
    pub created_at: i64,
    /// `None` means permanent, which is different from "no date known".
    pub expires_at: Option<i64>,
    pub active: bool,
    pub rule_code: Option<String>,
}

/// One rule from the project's rule book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleView {
    pub code: String,
    pub title: String,
    pub description: String,
    pub category: String,
    /// What breaking it costs. The point of reading the rules for most people,
    /// and the reason a rule book without them reads as a list of wishes.
    pub sanctions: Vec<SanctionView>,
}

/// One punishment a rule allows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionView {
    /// `warn` · `mute` · `ban` · `server_ban`.
    pub kind: String,
    pub label: String,
    /// Range in minutes. `None` on either side means "no bound" — and on the
    /// upper one that reads as permanent.
    pub min_minutes: Option<i64>,
    pub max_minutes: Option<i64>,
}

/// A support ticket in the list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketView {
    pub id: Uuid,
    /// Short form of the id, e.g. `a1b2c3d4`.
    ///
    /// A ticket has no number of its own on the master, and support answers
    /// with "which one?" to everything else. The first half of the uuid is
    /// stable, unique in practice and short enough to read out loud.
    pub number: String,
    pub subject: String,
    pub status: String,
    /// Staff replies the player has not opened.
    pub unread: i64,
    pub last_message_at: i64,
}

/// One message inside a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMessageView {
    pub author: String,
    /// Role the author held when they wrote, e.g. «Helper». Staff only.
    pub role: Option<String>,
    pub content: String,
    pub at: i64,
    /// Written by staff rather than the player.
    pub staff: bool,
}

/// A conversation in the list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmThreadView {
    pub peer_id: Uuid,
    pub peer_name: String,
    /// Where to fetch their head from. A bot has a picture of its own; a player
    /// gets one rendered from their skin.
    pub avatar_url: Option<String>,
    pub preview: String,
    pub unread: i64,
    pub last_message_at: i64,
}

/// One direct message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmMessageView {
    pub author_name: String,
    pub body: String,
    pub at: i64,
    /// Written by the player themselves.
    pub mine: bool,
}

/// An open conversation, with what the header needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmThreadOpen {
    pub peer: Uuid,
    pub peer_name: String,
    pub avatar_url: Option<String>,
    pub messages: Vec<DmMessageView>,
}
