//! noro-admin — CLI управления мастер-сервером.

mod client;
mod cmd_build;
mod cmd_cape;
mod cmd_core;
mod cmd_file;
mod cmd_info;
mod cmd_launcher;
mod cmd_mod;
mod cmd_news;
mod cmd_role;
mod cmd_server;
mod cmd_token;
mod cmd_user;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};
use client::Client;

#[derive(Parser)]
#[command(name = "noro-admin", about = "Управление noro мастер-сервером")]
struct Cli {
    /// URL мастера.
    #[arg(
        long,
        env = "NORO_MASTER_URL",
        default_value = "http://localhost:8080",
        global = true
    )]
    server: String,
    /// Admin-токен (или пользовательский Bearer с правами).
    #[arg(long, env = "NORO_ADMIN_TOKEN", default_value = "", global = true)]
    token: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Серверы и игровые сервера.
    Server {
        #[command(subcommand)]
        cmd: cmd_server::ServerCmd,
    },
    /// Сборки.
    Build {
        #[command(subcommand)]
        cmd: cmd_build::BuildCmd,
    },
    /// Файлы сборки (просмотр, загрузка, скачивание, редактирование, перемещение).
    File {
        #[command(subcommand)]
        cmd: cmd_file::FileCmd,
    },
    /// Поиск и добавление модов (Modrinth, CurseForge, URL).
    Mod {
        #[command(subcommand)]
        cmd: cmd_mod::ModCmd,
    },
    /// Роли и их права.
    Role {
        #[command(subcommand)]
        cmd: cmd_role::RoleCmd,
    },
    /// Пользователи (баны, роли, права, плащи).
    User {
        #[command(subcommand)]
        cmd: cmd_user::UserCmd,
    },
    /// Новости.
    News {
        #[command(subcommand)]
        cmd: cmd_news::NewsCmd,
    },
    /// Серверные ядра (JAR).
    Core {
        #[command(subcommand)]
        cmd: cmd_core::CoreCmd,
    },
    /// Admin-токены.
    Token {
        #[command(subcommand)]
        cmd: cmd_token::TokenCmd,
    },
    /// Версии лаунчера, сборка и деплой.
    Launcher {
        #[command(subcommand)]
        cmd: cmd_launcher::LauncherCmd,
    },
    /// Плащи.
    Cape {
        #[command(subcommand)]
        cmd: cmd_cape::CapeCmd,
    },
    /// Информация, статистика, права, агенты, версии.
    Info {
        #[command(subcommand)]
        cmd: cmd_info::InfoCmd,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let c = Client::new(cli.server, cli.token);

    match cli.command {
        Command::Server { cmd } => cmd_server::run(&c, cmd).await?,
        Command::Build { cmd } => cmd_build::run(&c, cmd).await?,
        Command::File { cmd } => cmd_file::run(&c, cmd).await?,
        Command::Mod { cmd } => cmd_mod::run(&c, cmd).await?,
        Command::Role { cmd } => cmd_role::run(&c, cmd).await?,
        Command::User { cmd } => cmd_user::run(&c, cmd).await?,
        Command::News { cmd } => cmd_news::run(&c, cmd).await?,
        Command::Core { cmd } => cmd_core::run(&c, cmd).await?,
        Command::Token { cmd } => cmd_token::run(&c, cmd).await?,
        Command::Launcher { cmd } => cmd_launcher::run(&c, cmd).await?,
        Command::Cape { cmd } => cmd_cape::run(&c, cmd).await?,
        Command::Info { cmd } => cmd_info::run(&c, cmd).await?,
    }
    Ok(())
}
