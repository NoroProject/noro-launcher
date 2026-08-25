//! Chat filter management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum ChatFilterCmd {
    /// List chat filter rules.
    List,
    /// Add a pattern to chat filter.
    Add {
        pattern: String,
        #[arg(long, default_value = "block")]
        action: String,
    },
    /// Delete a chat filter rule.
    Delete { id: String },
}

pub async fn run(c: &Client, cmd: ChatFilterCmd) -> Result<()> {
    match cmd {
        ChatFilterCmd::List => {
            let v = c.get("/api/admin/chat-filters").await?;
            print_json(&v);
        }
        ChatFilterCmd::Add { pattern, action } => {
            let body = serde_json::json!({ "pattern": pattern, "action": action });
            let v = c.post("/api/admin/chat-filters", body).await?;
            print_json(&v);
        }
        ChatFilterCmd::Delete { id } => {
            let path = format!("/api/admin/chat-filters/{}", urlencode(&id));
            let v = c.delete(&path).await?;
            print_json(&v);
        }
    }
    Ok(())
}
