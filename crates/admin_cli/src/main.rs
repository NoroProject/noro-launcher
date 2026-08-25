//! noro-admin — CLI управления мастер-сервером.

mod client;
mod format;
mod format_tables;
mod cmd_audit;
mod cmd_auth_methods;
mod cmd_backup;
mod cmd_blocklist;
mod cmd_build;
mod cmd_cape;
mod cmd_case;
mod cmd_chat_filter;
mod cmd_core;
mod cmd_file;
mod cmd_game;
mod cmd_info;
mod cmd_launcher;
mod cmd_mod;
mod cmd_news;
mod cmd_oauth;
mod cmd_punishment;
mod cmd_role;
mod cmd_server;
mod cmd_settings;
mod cmd_token;
mod cmd_user;
mod repl;
mod repl_completer;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};
use client::Client;

#[derive(Parser)]
#[command(name = "noro-admin", about = "Управление noro мастер-сервером")]
pub struct Cli {
    /// URL мастера.
    #[arg(long, env = "NORO_MASTER_URL", default_value = "http://localhost:8080", global = true)]
    pub server: String,
    /// Admin-токен (или пользовательский Bearer с правами).
    #[arg(long, env = "NORO_ADMIN_TOKEN", default_value = "", global = true)]
    pub token: String,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Interactive,
    Server { #[command(subcommand)] cmd: cmd_server::ServerCmd },
    Game { #[command(subcommand)] cmd: cmd_game::GameCmd },
    Exec { game_server_id: String, command: Vec<String> },
    Console { game_server_id: String },
    Power { game_server_id: String, action: String },
    Build { #[command(subcommand)] cmd: cmd_build::BuildCmd },
    File { #[command(subcommand)] cmd: cmd_file::FileCmd },
    Mod { #[command(subcommand)] cmd: cmd_mod::ModCmd },
    Role { #[command(subcommand)] cmd: cmd_role::RoleCmd },
    User { #[command(subcommand)] cmd: cmd_user::UserCmd },
    News { #[command(subcommand)] cmd: cmd_news::NewsCmd },
    Core { #[command(subcommand)] cmd: cmd_core::CoreCmd },
    Token { #[command(subcommand)] cmd: cmd_token::TokenCmd },
    Launcher { #[command(subcommand)] cmd: cmd_launcher::LauncherCmd },
    Cape { #[command(subcommand)] cmd: cmd_cape::CapeCmd },
    Info { #[command(subcommand)] cmd: cmd_info::InfoCmd },
    Audit { #[command(subcommand)] cmd: cmd_audit::AuditCmd },
    Backup { #[command(subcommand)] cmd: cmd_backup::BackupCmd },
    Punishment { #[command(subcommand)] cmd: cmd_punishment::PunishmentCmd },
    ChatFilter { #[command(subcommand)] cmd: cmd_chat_filter::ChatFilterCmd },
    Blocklist { #[command(subcommand)] cmd: cmd_blocklist::BlocklistCmd },
    Oauth { #[command(subcommand)] cmd: cmd_oauth::OauthCmd },
    AuthMethods { #[command(subcommand)] cmd: cmd_auth_methods::AuthMethodCmd },
    Case { #[command(subcommand)] cmd: cmd_case::CaseCmd },
    Settings { #[command(subcommand)] cmd: cmd_settings::SettingsCmd },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Interactive) | None => {
            repl::run_repl(cli.server, cli.token).await?;
        }
        Some(cmd) => {
            let c = Client::new(cli.server, cli.token);
            execute_command(&c, cmd).await?;
        }
    }
    Ok(())
}

pub async fn execute_command(c: &Client, cmd: Command) -> Result<()> {
    match cmd {
        Command::Interactive => {}
        Command::Game { cmd } => cmd_game::run(c, cmd).await?,
        Command::Exec { game_server_id, command } => {
            cmd_game::run(c, cmd_game::GameCmd::Exec { game_server_id, command }).await?;
        }
        Command::Console { game_server_id } => {
            cmd_game::run(c, cmd_game::GameCmd::Console { game_server_id }).await?;
        }
        Command::Power { game_server_id, action } => {
            cmd_game::run(c, cmd_game::GameCmd::Power { game_server_id, action }).await?;
        }
        Command::Server { cmd } => cmd_server::run(c, cmd).await?,
        Command::Build { cmd } => cmd_build::run(c, cmd).await?,
        Command::File { cmd } => cmd_file::run(c, cmd).await?,
        Command::Mod { cmd } => cmd_mod::run(c, cmd).await?,
        Command::Role { cmd } => cmd_role::run(c, cmd).await?,
        Command::User { cmd } => cmd_user::run(c, cmd).await?,
        Command::News { cmd } => cmd_news::run(c, cmd).await?,
        Command::Core { cmd } => cmd_core::run(c, cmd).await?,
        Command::Token { cmd } => cmd_token::run(c, cmd).await?,
        Command::Launcher { cmd } => cmd_launcher::run(c, cmd).await?,
        Command::Cape { cmd } => cmd_cape::run(c, cmd).await?,
        Command::Info { cmd } => cmd_info::run(c, cmd).await?,
        Command::Audit { cmd } => cmd_audit::run(c, cmd).await?,
        Command::Backup { cmd } => cmd_backup::run(c, cmd).await?,
        Command::Punishment { cmd } => cmd_punishment::run(c, cmd).await?,
        Command::ChatFilter { cmd } => cmd_chat_filter::run(c, cmd).await?,
        Command::Blocklist { cmd } => cmd_blocklist::run(c, cmd).await?,
        Command::Oauth { cmd } => cmd_oauth::run(c, cmd).await?,
        Command::AuthMethods { cmd } => cmd_auth_methods::run(c, cmd).await?,
        Command::Case { cmd } => cmd_case::run(c, cmd).await?,
        Command::Settings { cmd } => cmd_settings::run(c, cmd).await?,
    }
    Ok(())
}
