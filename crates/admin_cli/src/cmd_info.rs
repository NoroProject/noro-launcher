//! Info & lookup commands: stats, agents, permissions, versions.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum InfoCmd {
    /// Dashboard stats.
    Stats,
    /// List agent JARs.
    Agents,
    /// List available permission nodes.
    Permissions {
        #[arg(long)]
        server_id: Option<String>,
    },
    /// List Minecraft versions.
    McVersions,
    /// List modloader versions for a MC version.
    LoaderVersions {
        kind: String,
        #[arg(long)]
        mc: String,
    },
}

pub async fn run(c: &Client, cmd: InfoCmd) -> Result<()> {
    let v = match cmd {
        InfoCmd::Stats => c.get("/api/admin/stats").await?,
        InfoCmd::Agents => c.get("/api/admin/agents").await?,
        InfoCmd::Permissions { server_id } => {
            let path = match server_id {
                Some(id) => {
                    format!("/api/admin/permission-nodes?server_id={}", urlencode(&id))
                }
                None => "/api/admin/permission-nodes".to_string(),
            };
            c.get(&path).await?
        }
        InfoCmd::McVersions => c.get("/api/admin/versions/minecraft").await?,
        InfoCmd::LoaderVersions { kind, mc } => {
            c.get(&format!(
                "/api/admin/versions/loader/{}?mc={}",
                urlencode(&kind),
                urlencode(&mc)
            ))
            .await?
        }
    };
    print_json(&v);
    Ok(())
}
