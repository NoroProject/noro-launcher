//! Mod search and add commands.

use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum ModCmd {
    /// Search mods on Modrinth.
    Search {
        build_id: String,
        query: String,
        #[arg(long)]
        mc: Option<String>,
        #[arg(long)]
        loader: Option<String>,
    },
    /// List Modrinth project versions.
    ModrinthVersions {
        build_id: String,
        #[arg(long)]
        project_id: String,
        #[arg(long)]
        mc: Option<String>,
        #[arg(long)]
        loader: Option<String>,
    },
    /// Add mod from Modrinth by version ID.
    AddModrinth {
        build_id: String,
        version_id: String,
    },
    /// Add mod from CurseForge.
    AddCurseForge {
        build_id: String,
        project_id: u64,
        file_id: u64,
    },
    /// Add mod from direct URL.
    AddUrl { build_id: String, url: String },
}

pub async fn run(c: &Client, cmd: ModCmd) -> Result<()> {
    let v = match cmd {
        ModCmd::Search {
            build_id,
            query,
            mc,
            loader,
        } => {
            let mut path = format!(
                "/api/admin/builds/{build_id}/mods/search?q={}",
                urlencode(&query)
            );
            if let Some(mc) = mc {
                path.push_str(&format!("&mc={mc}"));
            }
            if let Some(l) = loader {
                path.push_str(&format!("&loader={l}"));
            }
            c.get(&path).await?
        }
        ModCmd::ModrinthVersions {
            build_id,
            project_id,
            mc,
            loader,
        } => {
            let mut path = format!(
                "/api/admin/builds/{build_id}/mods/modrinth-versions?project_id={}",
                urlencode(&project_id)
            );
            if let Some(mc) = mc {
                path.push_str(&format!("&mc={mc}"));
            }
            if let Some(l) = loader {
                path.push_str(&format!("&loader={l}"));
            }
            c.get(&path).await?
        }
        ModCmd::AddModrinth {
            build_id,
            version_id,
        } => {
            c.post(
                &format!("/api/admin/builds/{build_id}/mods/add-modrinth"),
                json!({ "version_id": version_id }),
            )
            .await?
        }
        ModCmd::AddCurseForge {
            build_id,
            project_id,
            file_id,
        } => {
            c.post(
                &format!("/api/admin/builds/{build_id}/mods/add-curseforge"),
                json!({ "project_id": project_id, "file_id": file_id }),
            )
            .await?
        }
        ModCmd::AddUrl { build_id, url } => {
            c.post(
                &format!("/api/admin/builds/{build_id}/mods/add-url"),
                json!({ "url": url }),
            )
            .await?
        }
    };
    print_json(&v);
    Ok(())
}
