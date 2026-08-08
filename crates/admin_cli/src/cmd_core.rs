//! Server core management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use serde_json::json;

#[derive(Subcommand)]
pub enum CoreCmd {
    /// List cores (optionally filter by server).
    List {
        #[arg(long)]
        server_id: Option<String>,
    },
    /// Upload a server core JAR.
    Upload {
        server_id: String,
        file: String,
        #[arg(long)]
        version: Option<String>,
    },
    /// Activate a core.
    Activate { id: String },
    /// Delete a core.
    Delete { id: String },
}

pub async fn run(c: &Client, cmd: CoreCmd) -> Result<()> {
    let v = match cmd {
        CoreCmd::List { server_id } => {
            let path = match server_id {
                Some(id) => format!("/api/admin/cores?server_id={id}"),
                None => "/api/admin/cores".to_string(),
            };
            c.get(&path).await?
        }
        CoreCmd::Upload {
            server_id,
            file,
            version,
        } => {
            let mut fields = vec![("server_id".to_string(), server_id)];
            if let Some(v) = version {
                fields.push(("version".to_string(), v));
            }
            c.upload_fields("/api/admin/cores", &file, fields).await?
        }
        CoreCmd::Activate { id } => {
            c.post(&format!("/api/admin/cores/{id}/activate"), json!({}))
                .await?
        }
        CoreCmd::Delete { id } => c.delete(&format!("/api/admin/cores/{id}")).await?,
    };
    print_json(&v);
    Ok(())
}
