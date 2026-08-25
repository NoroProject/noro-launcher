//! IP and HWID blocklist management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum BlocklistCmd {
    /// List blocked IPs / hardware IDs.
    List,
    /// Add an entry to blocklist.
    Add {
        value: String,
        #[arg(long, default_value = "ip")]
        kind: String,
        #[arg(long, default_value = "Blocked via admin CLI")]
        reason: String,
    },
    /// Remove an entry from blocklist.
    Delete { id: String },
}

pub async fn run(c: &Client, cmd: BlocklistCmd) -> Result<()> {
    match cmd {
        BlocklistCmd::List => {
            let v = c.get("/api/admin/blocklist").await?;
            print_json(&v);
        }
        BlocklistCmd::Add {
            value,
            kind,
            reason,
        } => {
            let body = serde_json::json!({ "value": value, "kind": kind, "reason": reason });
            let v = c.post("/api/admin/blocklist", body).await?;
            print_json(&v);
        }
        BlocklistCmd::Delete { id } => {
            let path = format!("/api/admin/blocklist/{}", urlencode(&id));
            let v = c.delete(&path).await?;
            print_json(&v);
        }
    }
    Ok(())
}
