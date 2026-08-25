//! Cape management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};

#[derive(Subcommand)]
pub enum CapeCmd {
    /// List all capes.
    List,
    /// Upload a cape image (file path and optional display name).
    Upload { file: String, name: Option<String> },
    /// Delete a cape.
    Delete { id: String },
}

pub async fn run(c: &Client, cmd: CapeCmd) -> Result<()> {
    let v = match cmd {
        CapeCmd::List => c.get("/api/admin/capes").await?,
        CapeCmd::Upload { file, name } => {
            let cape_name = name.unwrap_or_else(|| {
                std::path::Path::new(&file)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Cape".into())
            });
            c.upload_named_file(
                "/api/admin/capes",
                &file,
                "cape",
                vec![("name".to_string(), cape_name)],
            )
            .await?
        }
        CapeCmd::Delete { id } => c.delete(&format!("/api/admin/capes/{id}")).await?,
    };
    print_json(&v);
    Ok(())
}
