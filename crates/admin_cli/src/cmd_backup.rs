//! Server backups management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum BackupCmd {
    /// List backups.
    List,
    /// Create a new backup.
    Create {
        #[arg(long, default_value = "manual")]
        note: String,
    },
    /// Restore a backup.
    Restore { id: String },
}

pub async fn run(c: &Client, cmd: BackupCmd) -> Result<()> {
    match cmd {
        BackupCmd::List => {
            let v = c.get("/api/admin/backup").await?;
            print_json(&v);
        }
        BackupCmd::Create { note } => {
            let body = serde_json::json!({ "note": note });
            let v = c.post("/api/admin/backup", body).await?;
            print_json(&v);
        }
        BackupCmd::Restore { id } => {
            let path = format!("/api/admin/backup/{}/restore", urlencode(&id));
            let v = c.post(&path, serde_json::json!({})).await?;
            print_json(&v);
        }
    }
    Ok(())
}
