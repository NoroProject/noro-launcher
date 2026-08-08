//! Admin token management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use serde_json::json;

#[derive(Subcommand)]
pub enum TokenCmd {
    /// List admin tokens.
    List,
    /// Create an admin token.
    Create {
        name: String,
        #[arg(long)]
        perm: Vec<String>,
    },
    /// Revoke an admin token.
    Revoke { id: String },
}

pub async fn run(c: &Client, cmd: TokenCmd) -> Result<()> {
    let v = match cmd {
        TokenCmd::List => c.get("/api/admin/tokens").await?,
        TokenCmd::Create { name, perm } => {
            c.post(
                "/api/admin/tokens",
                json!({ "name": name, "permissions": perm }),
            )
            .await?
        }
        TokenCmd::Revoke { id } => c.delete(&format!("/api/admin/tokens/{id}")).await?,
    };
    print_json(&v);
    Ok(())
}
