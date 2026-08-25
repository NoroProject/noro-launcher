//! Live game server control, console execution, and power management.

use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

use crate::client::{print_json, Client};

#[derive(Subcommand)]
pub enum GameCmd {
    /// Execute a console command on a game server (e.g. "gamemode 1 Player", "op Alex").
    Exec {
        /// Game server ID or server name
        game_server_id: String,
        /// Command line to send to server console
        command: Vec<String>,
    },
    /// View console backlog and stream live console logs.
    Console {
        /// Game server ID or server name
        game_server_id: String,
    },
    /// Power management for game server (start | stop | restart | kill).
    Power {
        /// Game server ID or server name
        game_server_id: String,
        /// Action: start, stop, restart, kill
        action: String,
    },
    /// Kick a player from game.
    Kick {
        target: String,
        #[arg(long, default_value = "Kicked by administrator")]
        reason: String,
    },
    /// Send private message to player.
    Tell {
        target: String,
        message: String,
    },
    /// Broadcast announcement to all players.
    Announce {
        message: String,
    },
}

pub async fn run(c: &Client, cmd: GameCmd) -> Result<()> {
    match cmd {
        GameCmd::Exec { game_server_id, command } => {
            let gsid = resolve_game_server_id(c, &game_server_id).await?;
            let line = command.join(" ");
            if line.trim().is_empty() {
                anyhow::bail!("Command line cannot be empty");
            }
            let path = format!("/api/admin/game-servers/{gsid}/wrapper/command");
            let v = c.post(&path, json!({ "line": line })).await?;
            println!("\x1b[32mCommand sent to server console:\x1b[0m {}", line);
            print_json(&v);
        }
        GameCmd::Console { game_server_id } => {
            let gsid = resolve_game_server_id(c, &game_server_id).await?;
            let path = format!("/api/admin/game-servers/{gsid}/wrapper/console");
            let v = c.get(&path).await?;
            if let Some(lines) = v.get("lines").and_then(|l| l.as_array()) {
                println!("\x1b[1;33m--- Console Backlog ({}) ---\x1b[0m", lines.len());
                for line in lines {
                    if let Some(s) = line.as_str() {
                        println!("{s}");
                    }
                }
            } else {
                print_json(&v);
            }
        }
        GameCmd::Power { game_server_id, action } => {
            let gsid = resolve_game_server_id(c, &game_server_id).await?;
            let path = format!("/api/admin/game-servers/{gsid}/wrapper/power");
            let v = c.post(&path, json!({ "action": action })).await?;
            println!("\x1b[32mPower action '{}' sent to server {gsid}\x1b[0m", action);
            print_json(&v);
        }
        GameCmd::Kick { target, reason } => {
            let body = json!({ "target": target, "message": reason });
            let v = c.post("/api/admin/game/kick", body).await?;
            print_json(&v);
        }
        GameCmd::Tell { target, message } => {
            let body = json!({ "target": target, "message": message });
            let v = c.post("/api/admin/game/tell", body).await?;
            print_json(&v);
        }
        GameCmd::Announce { message } => {
            let body = json!({ "message": message });
            let v = c.post("/api/admin/game/announce", body).await?;
            print_json(&v);
        }
    }
    Ok(())
}

async fn resolve_game_server_id(c: &Client, input: &str) -> Result<String> {
    if uuid::Uuid::parse_str(input).is_ok() {
        return Ok(input.to_string());
    }
    if let Ok(v) = c.get("/api/admin/servers").await {
        if let Some(arr) = v.as_array().or_else(|| v.get("items").and_then(|i| i.as_array())) {
            for item in arr {
                let name = item.get("name").and_then(|s| s.as_str()).unwrap_or("");
                if name.eq_ignore_ascii_case(input) {
                    if let Some(gs_arr) = item.get("game_servers").and_then(|g| g.as_array()) {
                        if let Some(first_gs) = gs_arr.first() {
                            if let Some(id) = first_gs.get("id").and_then(|s| s.as_str()) {
                                return Ok(id.to_string());
                            }
                        }
                    }
                    if let Some(id) = item.get("id").and_then(|s| s.as_str()) {
                        return Ok(id.to_string());
                    }
                }
            }
        }
    }
    Ok(input.to_string())
}
