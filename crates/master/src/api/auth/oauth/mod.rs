//! Вход через внешние платформы: Discord, Twitch, Google.
//!
//! Раньше здесь был один Discord, вшитый в колонки `users`. Теперь платформа —
//! это привязка в `user_identities`, их у аккаунта может быть несколько, и
//! войти можно любой.

pub mod config;
pub mod flow;
pub mod link;
pub mod methods;
pub mod provider;
pub mod remote;
pub mod states;

pub use provider::Provider;
