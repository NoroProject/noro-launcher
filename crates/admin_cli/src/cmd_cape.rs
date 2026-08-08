//! Cape management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};

#[derive(Subcommand)]
pub enum CapeCmd {
    /// List all capes.
    List,
    /// Upload a cape image.
    Upload { file: String },
    /// Delete a cape.
    Delete { id: String },
}

pub async fn run(c: &Client, cmd: CapeCmd) -> Result<()> {
    let v = match cmd {
        CapeCmd::List => c.get("/api/admin/capes").await?,
        CapeCmd::Upload { file } => c.upload_image("/api/admin/capes", &file, "file").await?,
        CapeCmd::Delete { id } => c.delete(&format!("/api/admin/capes/{id}")).await?,
    };
    print_json(&v);
    Ok(())
}
