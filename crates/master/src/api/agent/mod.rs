//! API для серверных агентов (плагин Paper, моды NeoForge и Fabric).
//!
//! Агент живёт на игровом сервере и знает игрока только по MC UUID. Мастер —
//! источник истины по ролям, доступу и наказаниям; сервер получается его
//! проекцией, а не второй независимой базой.
//!
//! Авторизация — секретом конкретного игрового сервера, а не админ-токеном:
//! сервер должен уметь спросить только про себя. Из секрета же берётся, на
//! какой сервер заходит игрок, поэтому подменить его в запросе нельзя.

pub mod heartbeat;
pub mod history;
pub mod player;
pub mod punish;
pub mod types;

pub use heartbeat::heartbeat;
pub use history::{acknowledge, list_punishments, revoke_punishment};
pub use player::{player, player_by_name};
pub use punish::create_punishment;
