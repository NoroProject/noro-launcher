use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

use crate::client::{print_json, Client};

#[derive(Subcommand)]
pub enum BuildCmd {
    /// List builds for a server.
    List { server_id: String },
    /// Get build details.
    Get { id: String },
    /// Create a build.
    Create {
        server_id: String,
        version: String,
        modloader: String,
        mc_version: String,
        #[arg(long)]
        modloader_version: Option<String>,
    },
    /// Publish a build (bootstrap + sign).
    Publish { id: String },
    /// Unpublish a build.
    Unpublish { id: String },
    /// Rebuild manifest.
    Rebuild { id: String },
    /// Rebuild manifest (clean, re-bootstrap all).
    RebuildClean { id: String },
    /// Delete a build.
    Delete { id: String },
    /// Set MC/modloader versions.
    SetVersions {
        id: String,
        #[arg(long)]
        mc_version: Option<String>,
        #[arg(long)]
        modloader: Option<String>,
        #[arg(long)]
        modloader_version: Option<String>,
    },
    /// Set unmanaged/user-managed paths.
    SetPaths {
        id: String,
        #[arg(long = "unmanaged")]
        unmanaged: Vec<String>,
        #[arg(long = "user-managed")]
        user_managed: Vec<String>,
    },
    /// Set recommended JVM settings.
    SetSettings {
        id: String,
        #[arg(long, default_value_t = 2048)]
        min_mb: u32,
        #[arg(long, default_value_t = 4096)]
        max_mb: u32,
        #[arg(long, default_value = "")]
        jvm_flags: String,
        #[arg(long, default_value_t = false)]
        show_console: bool,
    },
    /// Import .mrpack file.
    ImportMrpack { id: String, file: String },
    /// Import CurseForge zip.
    ImportCf { id: String, file: String },
    /// Import instance zip.
    ImportZip { id: String, file: String },
    /// Check import job progress.
    ImportProgress { id: String, job_id: String },
    /// List optional mods.
    OptionalMods { id: String },
}

pub async fn run(c: &Client, cmd: BuildCmd) -> Result<()> {
    let v = match cmd {
        BuildCmd::List { server_id } => c.get(&format!("/api/admin/builds?server_id={server_id}")).await?,
        BuildCmd::Get { id } => c.get(&format!("/api/admin/builds/{id}")).await?,
        BuildCmd::Create { server_id, version, modloader, mc_version, modloader_version } => {
            c.post("/api/admin/builds", json!({
                "server_id": server_id, "version": version, "modloader": modloader,
                "mc_version": mc_version, "modloader_version": modloader_version
            })).await?
        }
        BuildCmd::Publish { id } => {
            eprintln!("publishing (bootstrap may take a while)...");
            c.post(&format!("/api/admin/builds/{id}/publish"), json!({})).await?
        }
        BuildCmd::Unpublish { id } => c.post(&format!("/api/admin/builds/{id}/unpublish"), json!({})).await?,
        BuildCmd::Rebuild { id } => c.post(&format!("/api/admin/builds/{id}/rebuild"), json!({})).await?,
        BuildCmd::RebuildClean { id } => c.post(&format!("/api/admin/builds/{id}/rebuild-clean"), json!({})).await?,
        BuildCmd::Delete { id } => c.delete(&format!("/api/admin/builds/{id}")).await?,
        BuildCmd::SetVersions { id, mc_version, modloader, modloader_version } => {
            c.put(&format!("/api/admin/builds/{id}/versions"), json!({
                "mc_version": mc_version, "modloader": modloader, "modloader_version": modloader_version
            })).await?
        }
        BuildCmd::SetPaths { id, unmanaged, user_managed } => {
            c.put(&format!("/api/admin/builds/{id}/paths"), json!({
                "unmanaged_paths": unmanaged, "user_managed_paths": user_managed
            })).await?
        }
        BuildCmd::SetSettings { id, min_mb, max_mb, jvm_flags, show_console } => {
            c.put(&format!("/api/admin/builds/{id}/recommended-settings"), json!({
                "memory_min_mb": min_mb, "memory_max_mb": max_mb,
                "jvm_flags": jvm_flags, "show_console_on_launch": show_console
            })).await?
        }
        BuildCmd::ImportMrpack { id, file } => {
            c.upload(&format!("/api/admin/builds/{id}/import/mrpack"), &file, None).await?
        }
        BuildCmd::ImportCf { id, file } => {
            c.upload(&format!("/api/admin/builds/{id}/import/curseforge"), &file, None).await?
        }
        BuildCmd::ImportZip { id, file } => {
            c.upload(&format!("/api/admin/builds/{id}/import/zip"), &file, None).await?
        }
        BuildCmd::ImportProgress { id, job_id } => {
            c.get(&format!("/api/admin/builds/{id}/import-progress/{job_id}")).await?
        }
        BuildCmd::OptionalMods { id } => c.get(&format!("/api/admin/builds/{id}/optional-mods")).await?,
    };
    print_json(&v);
    Ok(())
}
