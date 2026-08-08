//! Launcher version management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use serde_json::json;

#[derive(Subcommand)]
pub enum LauncherCmd {
    /// List launcher versions.
    Versions,
    /// Get latest GitHub release info.
    Github,
    /// Trigger a launcher build from a Git tag.
    Build { tag: String },
    /// List active build jobs.
    BuildJobs,
    /// Get build job log.
    Log { job_id: String },
    /// Deploy a launcher version.
    Deploy { version_id: String },
}

pub async fn run(c: &Client, cmd: LauncherCmd) -> Result<()> {
    let v = match cmd {
        LauncherCmd::Versions => c.get("/api/admin/launcher/versions").await?,
        LauncherCmd::Github => c.get("/api/admin/launcher/github").await?,
        LauncherCmd::Build { tag } => {
            c.post("/api/admin/launcher/build", json!({ "tag": tag }))
                .await?
        }
        LauncherCmd::BuildJobs => c.get("/api/admin/launcher/builds").await?,
        LauncherCmd::Log { job_id } => {
            c.get(&format!("/api/admin/launcher/build/{job_id}/log"))
                .await?
        }
        LauncherCmd::Deploy { version_id } => {
            c.post(
                &format!("/api/admin/launcher/deploy/{version_id}"),
                json!({}),
            )
            .await?
        }
    };
    print_json(&v);
    Ok(())
}
