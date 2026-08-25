//! Auth Methods management commands for setting up Telegram, Discord, Twitch, Google, and Passkey.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum AuthMethodCmd {
    /// List available authentication methods and their status.
    List,
    /// Configure an authentication method (e.g., telegram, discord, twitch, google).
    Set {
        /// Method name (e.g. telegram, discord, twitch, google)
        method: String,
        /// Client ID / Bot Username (e.g. @MyBot for Telegram or client_id)
        #[arg(long, default_value = "")]
        client_id: String,
        /// Client Secret / Bot Token (e.g. 123456:ABC-DEF... for Telegram)
        #[arg(long)]
        client_secret: Option<String>,
        /// Enable this authentication method
        #[arg(long)]
        enable: bool,
        /// Disable this authentication method
        #[arg(long)]
        disable: bool,
    },
}

pub async fn run(c: &Client, cmd: AuthMethodCmd) -> Result<()> {
    match cmd {
        AuthMethodCmd::List => {
            let v = c.get("/api/admin/auth-methods").await?;
            print_json(&v);
        }
        AuthMethodCmd::Set {
            method,
            client_id,
            client_secret,
            enable,
            disable,
        } => {
            let enabled = !disable || enable;
            let body = serde_json::json!({
                "client_id": client_id,
                "client_secret": client_secret,
                "enabled": enabled,
            });
            let path = format!("/api/admin/auth-methods/{}", urlencode(&method));
            let v = c.put(&path, body).await?;
            print_json(&v);
        }
    }
    Ok(())
}
