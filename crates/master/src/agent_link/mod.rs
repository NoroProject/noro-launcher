//! Живой канал до агентов игровых серверов: наказания применяются сразу, а не
//! к следующему входу игрока.

pub mod hub;
pub mod inbox;
pub mod notify;
pub mod proto;
pub mod roster;
pub mod session;

pub use hub::AgentHub;
pub use roster::Roster;
