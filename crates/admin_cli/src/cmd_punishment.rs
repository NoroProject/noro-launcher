//! Punishments (bans, mutes, warns) management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum PunishmentCmd {
    /// List active or historical punishments.
    List {
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Revoke / pardon a punishment.
    Revoke {
        id: String,
        #[arg(long, default_value = "Revoked by admin CLI")]
        reason: String,
    },
}

pub async fn run(c: &Client, cmd: PunishmentCmd) -> Result<()> {
    match cmd {
        PunishmentCmd::List { user_id, limit } => {
            let path = match user_id {
                Some(uid) => format!(
                    "/api/admin/punishments?user_id={}&limit={limit}",
                    urlencode(&uid)
                ),
                None => format!("/api/admin/punishments?limit={limit}"),
            };
            let v = c.get(&path).await?;
            print_json(&v);
        }
        PunishmentCmd::Revoke { id, reason } => {
            let path = format!("/api/admin/punishments/{}/revoke", urlencode(&id));
            let body = serde_json::json!({ "reason": reason });
            let v = c.post(&path, body).await?;
            print_json(&v);
        }
    }
    Ok(())
}
