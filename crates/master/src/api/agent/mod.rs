//! API for game-server agents (the Paper plugin, the NeoForge and Fabric mods).
//!
//! An agent knows a player only by MC UUID; the master stays the source of
//! truth for roles, access and punishments.
//!
//! Auth is the individual game server's secret rather than the admin token, so
//! a server can only ask about itself. Which server a player is joining is
//! taken from that secret too, so a request can't claim to be elsewhere.

pub mod chat_filters;
pub mod heartbeat;
pub mod history;
pub mod player;
pub mod prefix_pack;
pub mod punish;
pub mod reports;
pub mod types;

pub use chat_filters::{list as list_chat_filters, record_trigger as record_automod_trigger};
pub use heartbeat::heartbeat;
pub use history::{acknowledge, list_punishments, revoke_punishment};
pub use player::{player, player_by_name, players_batch};
pub use punish::create_punishment;
pub use reports::create_report as agent_create_report;
