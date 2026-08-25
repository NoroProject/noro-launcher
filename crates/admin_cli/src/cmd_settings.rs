//! Global server settings and maintenance mode commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};

#[derive(Subcommand)]
pub enum SettingsCmd {
    /// Get all global settings.
    Get,
    /// Enable or disable maintenance mode.
    Maintenance { enabled: bool },
    /// Rebuild prefix pack.
    RebuildPrefixes,
}

pub async fn run(c: &Client, cmd: SettingsCmd) -> Result<()> {
    match cmd {
        SettingsCmd::Get => {
            let v = c.get("/api/admin/settings").await?;
            print_json(&v);
        }
        SettingsCmd::Maintenance { enabled } => {
            let body = serde_json::json!({ "maintenance_mode": enabled });
            let v = c.post("/api/admin/settings", body).await?;
            print_json(&v);
        }
        SettingsCmd::RebuildPrefixes => {
            let v = c.post("/api/admin/prefix-pack/rebuild", serde_json::json!({})).await?;
            print_json(&v);
        }
    }
    Ok(())
}
