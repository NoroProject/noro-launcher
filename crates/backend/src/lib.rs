//! Backend лаунчера: вся сетевая/файловая/игровая логика на tokio.
//!
//! Связан с frontend через [`bridge`]. Точка входа — [`backend::start`].

pub mod auth;
pub mod backend;
pub mod backend_handler;
pub mod catalog_search;
pub mod config;
pub mod diagnostics;
pub mod directories;
pub mod discord_rpc;
pub mod game_runner;
pub mod impersonation;
pub mod log_reader;
pub mod mod_icon;
pub mod mod_link;
pub mod persistent;
pub mod remote_actions;
pub mod servers_dat;
pub mod signing;
pub mod support;
pub mod sync;
pub mod telemetry;
pub mod translations;
pub mod updater;
pub mod ws_client;

pub use backend::start;
pub use directories::LauncherDirectories;
