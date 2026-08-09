//! Launcher pages facade.

mod common;
mod game;
mod game_bar;
pub mod game_console;
mod game_empty;
mod game_status;
mod game_sync;
mod markdown;
mod mod_icon;
mod news;
mod news_detail;
mod profile;
mod profile_asset;
mod profile_skin;
mod profile_user;
mod server_mod_catalog;
mod server_mods;
mod server_settings;
mod settings;
mod settings_panel;
mod settings_rows;
mod shell;
mod sidebar;
mod sidebar_parts;
mod sidebar_server;
mod sidebar_user;
mod skin_drag;
mod toast;

pub use shell::launcher_shell;
pub use toast::toast_overlay;
