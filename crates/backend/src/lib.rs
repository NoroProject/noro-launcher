//! Backend лаунчера: вся сетевая/файловая/игровая логика на tokio.
//!
//! Связан с frontend через [`bridge`]. Точка входа — [`backend::start`].

pub mod auth;
pub mod backend;
pub mod backend_handler;
pub mod config;
pub mod directories;
pub mod game_runner;
pub mod log_reader;
pub mod mod_icon;
pub mod persistent;
pub mod servers_dat;
pub mod signing;
pub mod sync;
pub mod translations;
pub mod updater;
pub mod ws_client;

pub use backend::start;
pub use directories::LauncherDirectories;
