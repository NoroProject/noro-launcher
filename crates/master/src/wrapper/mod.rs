//! Управление игровыми серверами через ServerWrapper.
//!
//! Враппер живёт на машине игрового сервера и сам приходит к мастеру по
//! WebSocket — обратного пути нет и не нужно. Здесь реестр подключений,
//! протокол и операции, которые админка запускает поверх него.

pub mod fs_ops;
pub mod hub;
#[cfg(test)]
mod hub_tests;
pub mod ops;
pub mod proto;
pub mod session;

pub use hub::WrapperHub;
pub use proto::{Op, WrapperState};
