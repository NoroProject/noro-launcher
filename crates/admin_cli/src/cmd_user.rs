//! User management commands with username, role resolution, skin, and skin preset management.

use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum UserCmd {
    /// List all users.
    List,
    /// Get user details.
    Get { id: String },
    /// Ban a user.
    Ban { id: String, #[arg(long)] reason: Option<String> },
    /// Unban a user.
    Unban { id: String },
    /// Add role to user (alias: set-role).
    #[command(alias = "set-role")]
    AddRole { id: String, role_id: String },
    /// Remove role from user.
    RemoveRole { id: String, role_id: String },
    /// Add permission to user (alias: set-perm).
    #[command(alias = "set-perm")]
    AddPerm { id: String, permission: String },
    /// Remove permission from user.
    RemovePerm { id: String, permission: String },
    /// Set user's cape.
    SetCape { id: String, cape_id: String },
    /// Upload custom skin PNG for user.
    SetSkin { id: String, file: String },
    /// Fetch official Minecraft skin from Mojang/NameMC and set for user.
    FetchSkin { id: String, mc_username: Option<String> },
    /// List user's skin presets (alias: presets).
    #[command(alias = "presets")]
    SkinPresets { id: String },
    /// Save current or specified skin into user's presets (alias: add-preset).
    #[command(alias = "add-preset")]
    SavePreset { id: String, #[arg(long)] name: Option<String>, #[arg(long)] skin_url: Option<String>, #[arg(long)] slim: Option<bool> },
    /// Select a skin preset for user (alias: select-preset).
    #[command(alias = "select-preset")]
    SelectPreset { id: String, skin_url: String, #[arg(long, default_value_t = false)] slim: bool },
    /// Delete a skin preset for user (alias: delete-preset).
    #[command(alias = "delete-preset")]
    DeletePreset { id: String, preset_id: String },
}

pub async fn run(c: &Client, cmd: UserCmd) -> Result<()> {
    let v = match cmd {
        UserCmd::List => c.get("/api/admin/users").await?,
        UserCmd::Get { id } => {
            let uid = resolve_user_id(c, &id).await?;
            c.get(&format!("/api/admin/users/{uid}")).await?
        }
        UserCmd::Ban { id, reason } => {
            let uid = resolve_user_id(c, &id).await?;
            c.put(&format!("/api/admin/users/{uid}/ban"), json!({ "banned": true, "reason": reason })).await?
        }
        UserCmd::Unban { id } => {
            let uid = resolve_user_id(c, &id).await?;
            c.put(&format!("/api/admin/users/{uid}/ban"), json!({ "banned": false })).await?
        }
        UserCmd::AddRole { id, role_id } => {
            let uid = resolve_user_id(c, &id).await?;
            let rid = resolve_role_id(c, &role_id).await?;
            c.post(&format!("/api/admin/users/{uid}/roles/{rid}"), json!({})).await?
        }
        UserCmd::RemoveRole { id, role_id } => {
            let uid = resolve_user_id(c, &id).await?;
            let rid = resolve_role_id(c, &role_id).await?;
            c.delete(&format!("/api/admin/users/{uid}/roles/{rid}")).await?
        }
        UserCmd::AddPerm { id, permission } => {
            let uid = resolve_user_id(c, &id).await?;
            c.post(&format!("/api/admin/users/{uid}/permissions"), json!({ "permission": permission })).await?
        }
        UserCmd::RemovePerm { id, permission } => {
            let uid = resolve_user_id(c, &id).await?;
            c.delete(&format!("/api/admin/users/{uid}/permissions/{}", urlencode(&permission))).await?
        }
        UserCmd::SetCape { id, cape_id } => {
            let uid = resolve_user_id(c, &id).await?;
            c.put(&format!("/api/admin/users/{uid}/cape"), json!({ "cape_id": cape_id })).await?
        }
        UserCmd::SetSkin { id, file } => {
            let uid = resolve_user_id(c, &id).await?;
            c.upload_image(&format!("/api/admin/users/{uid}/skin"), &file, "skin").await?
        }
        UserCmd::FetchSkin { id, mc_username } => {
            let uid = resolve_user_id(c, &id).await?;
            let target_name = mc_username.unwrap_or_else(|| id.clone());
            let profile_url = format!("https://api.mojang.com/users/profiles/minecraft/{target_name}");
            let resp: serde_json::Value = reqwest::get(&profile_url).await?.json().await?;
            let uuid_hex = resp.get("id").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Player not found on Mojang API"))?;
            let sess_url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid_hex}");
            let sess: serde_json::Value = reqwest::get(&sess_url).await?.json().await?;
            let mut skin_url: Option<String> = None;
            if let Some(props) = sess.get("properties").and_then(|p| p.as_array()) {
                for p in props {
                    if p.get("name").and_then(|s| s.as_str()) == Some("textures") {
                        if let Some(val_b64) = p.get("value").and_then(|s| s.as_str()) {
                            use base64::Engine;
                            let decoded = base64::engine::general_purpose::STANDARD.decode(val_b64)?;
                            let tex_json: serde_json::Value = serde_json::from_slice(&decoded)?;
                            skin_url = tex_json.get("textures").and_then(|t| t.get("SKIN")).and_then(|s| s.get("url")).and_then(|u| u.as_str()).map(String::from);
                        }
                    }
                }
            }
            let skin_url = skin_url.ok_or_else(|| anyhow::anyhow!("No skin texture found for player"))?;
            let skin_bytes = reqwest::get(&skin_url).await?.bytes().await?;
            let temp_path = format!("/tmp/skin_{uuid_hex}.png");
            tokio::fs::write(&temp_path, &skin_bytes).await?;
            let res = c.upload_image(&format!("/api/admin/users/{uid}/skin"), &temp_path, "skin").await?;
            let _ = tokio::fs::remove_file(&temp_path).await;
            res
        }
        UserCmd::SkinPresets { id } => {
            let uid = resolve_user_id(c, &id).await?;
            c.get(&format!("/api/admin/users/{uid}/skin-presets")).await?
        }
        UserCmd::SavePreset { id, name, skin_url, slim } => {
            let uid = resolve_user_id(c, &id).await?;
            c.post(&format!("/api/admin/users/{uid}/skin-presets"), json!({ "name": name, "skin_url": skin_url, "slim": slim })).await?
        }
        UserCmd::SelectPreset { id, skin_url, slim } => {
            let uid = resolve_user_id(c, &id).await?;
            c.post(&format!("/api/admin/users/{uid}/skin-presets/select"), json!({ "skin_url": skin_url, "slim": slim })).await?
        }
        UserCmd::DeletePreset { id, preset_id } => {
            let uid = resolve_user_id(c, &id).await?;
            c.delete(&format!("/api/admin/users/{uid}/skin-presets/{preset_id}")).await?
        }
    };
    print_json(&v);
    Ok(())
}

async fn resolve_user_id(c: &Client, input: &str) -> Result<String> {
    if uuid::Uuid::parse_str(input).is_ok() { return Ok(input.to_string()); }
    let v = c.get("/api/admin/users").await?;
    if let Some(arr) = v.as_array().or_else(|| v.get("items").and_then(|i| i.as_array())) {
        for u in arr {
            let username = u.get("username").and_then(|s| s.as_str()).unwrap_or("");
            if username.eq_ignore_ascii_case(input) {
                if let Some(id) = u.get("id").and_then(|s| s.as_str()) { return Ok(id.to_string()); }
            }
        }
    }
    Ok(input.to_string())
}

async fn resolve_role_id(c: &Client, input: &str) -> Result<String> {
    if uuid::Uuid::parse_str(input).is_ok() { return Ok(input.to_string()); }
    let v = c.get("/api/admin/roles").await?;
    if let Some(arr) = v.as_array().or_else(|| v.get("items").and_then(|i| i.as_array())) {
        for r in arr {
            let name = r.get("name").and_then(|s| s.as_str()).unwrap_or("");
            if name.eq_ignore_ascii_case(input) {
                if let Some(id) = r.get("id").and_then(|s| s.as_str()) { return Ok(id.to_string()); }
            }
        }
    }
    Ok(input.to_string())
}
