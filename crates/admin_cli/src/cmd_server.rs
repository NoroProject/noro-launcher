use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

use crate::client::{print_json, Client};

#[derive(Subcommand)]
pub enum ServerCmd {
    /// List all servers.
    List,
    /// Get server details.
    Get { id: String },
    /// Create a server.
    Create {
        name: String,
        modloader: String,
        mc_version: String,
    },
    /// Update server.
    Edit {
        id: String,
        name: String,
        description: String,
        modloader: String,
        mc_version: String,
        #[arg(long, default_value_t = true)]
        active: bool,
        #[arg(long, default_value_t = false)]
        limited: bool,
        #[arg(long, default_value_t = 0)]
        sort_order: i32,
    },
    /// Delete a server.
    Delete { id: String },
    /// Reorder servers.
    Reorder { ids: Vec<String> },
    /// Upload server icon.
    UploadIcon { id: String, file: String },
    /// Upload server background.
    UploadBg { id: String, file: String },
    /// Game-server subcommands.
    GameServer {
        #[command(subcommand)]
        cmd: GameServerCmd,
    },
}

#[derive(Subcommand)]
pub enum GameServerCmd {
    /// List game servers of a server.
    List { server_id: String },
    /// Create game server.
    Create {
        server_id: String,
        name: String,
        #[arg(long, default_value = "")]
        mc_host: String,
        #[arg(long, default_value_t = 25565)]
        mc_port: i32,
        #[arg(long)]
        kind: Option<String>,
    },
    /// Update game server.
    Update {
        server_id: String,
        id: String,
        name: String,
        #[arg(long, default_value = "")]
        mc_host: String,
        #[arg(long, default_value_t = 25565)]
        mc_port: i32,
        #[arg(long, default_value_t = 0)]
        sort_order: i32,
        #[arg(long)]
        kind: Option<String>,
    },
    /// Delete game server.
    Delete { server_id: String, id: String },
    /// Rotate game server token.
    RotateToken { server_id: String, id: String },
}

pub async fn run(c: &Client, cmd: ServerCmd) -> Result<()> {
    let v = match cmd {
        ServerCmd::List => c.get("/api/admin/servers").await?,
        ServerCmd::Get { id } => c.get(&format!("/api/admin/servers/{id}")).await?,
        ServerCmd::Create { name, modloader, mc_version } => {
            c.post("/api/admin/servers", json!({ "name": name, "modloader": modloader, "mc_version": mc_version })).await?
        }
        ServerCmd::Edit { id, name, description, modloader, mc_version, active, limited, sort_order } => {
            c.put(&format!("/api/admin/servers/{id}"), json!({
                "name": name, "description": description, "modloader": modloader,
                "mc_version": mc_version, "active": active, "limited": limited, "sort_order": sort_order
            })).await?
        }
        ServerCmd::Delete { id } => c.delete(&format!("/api/admin/servers/{id}")).await?,
        ServerCmd::Reorder { ids } => c.put("/api/admin/servers/reorder", json!({ "order": ids })).await?,
        ServerCmd::UploadIcon { id, file } => c.upload_image(&format!("/api/admin/servers/{id}/icon"), &file, "image").await?,
        ServerCmd::UploadBg { id, file } => c.upload_image(&format!("/api/admin/servers/{id}/background"), &file, "image").await?,
        ServerCmd::GameServer { cmd } => return game_server(c, cmd).await,
    };
    print_json(&v);
    Ok(())
}

async fn game_server(c: &Client, cmd: GameServerCmd) -> Result<()> {
    let v = match cmd {
        GameServerCmd::List { server_id } => c.get(&format!("/api/admin/servers/{server_id}/game-servers")).await?,
        GameServerCmd::Create { server_id, name, mc_host, mc_port, kind } => {
            c.post(&format!("/api/admin/servers/{server_id}/game-servers"),
                json!({ "name": name, "mc_host": mc_host, "mc_port": mc_port, "kind": kind })).await?
        }
        GameServerCmd::Update { server_id, id, name, mc_host, mc_port, sort_order, kind } => {
            c.put(&format!("/api/admin/servers/{server_id}/game-servers/{id}"),
                json!({ "name": name, "mc_host": mc_host, "mc_port": mc_port, "sort_order": sort_order, "kind": kind })).await?
        }
        GameServerCmd::Delete { server_id, id } => c.delete(&format!("/api/admin/servers/{server_id}/game-servers/{id}")).await?,
        GameServerCmd::RotateToken { server_id, id } => {
            c.post(&format!("/api/admin/servers/{server_id}/game-servers/{id}/token"), json!({})).await?
        }
    };
    print_json(&v);
    Ok(())
}
