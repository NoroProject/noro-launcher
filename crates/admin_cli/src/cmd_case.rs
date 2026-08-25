//! Moderation cases and reports commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum CaseCmd {
    /// List moderation cases.
    List {
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Get details of a moderation case.
    Get { id: String },
    /// Close a moderation case.
    Close {
        id: String,
        #[arg(long, default_value = "Resolved")]
        resolution: String,
    },
}

pub async fn run(c: &Client, cmd: CaseCmd) -> Result<()> {
    match cmd {
        CaseCmd::List { limit } => {
            let v = c.get(&format!("/api/admin/cases?limit={limit}")).await?;
            print_json(&v);
        }
        CaseCmd::Get { id } => {
            let v = c
                .get(&format!("/api/admin/cases/{}", urlencode(&id)))
                .await?;
            print_json(&v);
        }
        CaseCmd::Close { id, resolution } => {
            let path = format!("/api/admin/cases/{}/close", urlencode(&id));
            let body = serde_json::json!({ "resolution": resolution });
            let v = c.post(&path, body).await?;
            print_json(&v);
        }
    }
    Ok(())
}
