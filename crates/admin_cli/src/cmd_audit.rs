//! Audit logs commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};

#[derive(Subcommand)]
pub enum AuditCmd {
    /// List audit log records.
    List {
        #[arg(long, default_value = "50")]
        limit: u32,
        #[arg(long, default_value = "0")]
        offset: u32,
    },
}

pub async fn run(c: &Client, cmd: AuditCmd) -> Result<()> {
    match cmd {
        AuditCmd::List { limit, offset } => {
            let path = format!("/api/admin/audit?limit={limit}&offset={offset}");
            let v = c.get(&path).await?;
            print_json(&v);
        }
    }
    Ok(())
}
