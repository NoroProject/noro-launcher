//! API для серверных агентов (плагин Paper, моды NeoForge и Fabric).
//!
//! Агент живёт на игровом сервере и знает игрока только по MC UUID. Мастер —
//! источник истины по ролям, доступу и наказаниям; сервер получается его
//! проекцией, а не второй независимой базой.
//!
//! Авторизация — секретом конкретного игрового сервера, а не админ-токеном:
//! сервер должен уметь спросить только про себя. Из секрета же берётся, на
//! какой сервер заходит игрок, поэтому подменить его в запросе нельзя.

pub mod chat_filters;
pub mod heartbeat;
pub mod history;
pub mod player;
pub mod punish;
pub mod reports;
pub mod types;

pub use chat_filters::{list as list_chat_filters, record_trigger as record_automod_trigger};
pub use heartbeat::heartbeat;
pub use history::{acknowledge, list_punishments, revoke_punishment};
pub use player::{player, player_by_name, players_batch};
pub use punish::create_punishment;
pub use reports::create_report as agent_create_report;
