//! User management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;
use serde_json::json;

#[derive(Subcommand)]
pub enum UserCmd {
    /// List all users.
    List,
    /// Get user details.
    Get { id: String },
    /// Ban a user.
    Ban {
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Unban a user.
    Unban { id: String },
    /// Add role to user.
    AddRole { id: String, role_id: String },
    /// Remove role from user.
    RemoveRole { id: String, role_id: String },
    /// Add permission to user.
    AddPerm { id: String, permission: String },
    /// Remove permission from user.
    RemovePerm { id: String, permission: String },
    /// Set user's cape.
    SetCape { id: String, cape_id: String },
}

pub async fn run(c: &Client, cmd: UserCmd) -> Result<()> {
    let v = match cmd {
        UserCmd::List => c.get("/api/admin/users").await?,
        UserCmd::Get { id } => c.get(&format!("/api/admin/users/{id}")).await?,
        UserCmd::Ban { id, reason } => {
            c.put(
                &format!("/api/admin/users/{id}/ban"),
                json!({ "banned": true, "reason": reason }),
            )
            .await?
        }
        UserCmd::Unban { id } => {
            c.put(
                &format!("/api/admin/users/{id}/ban"),
                json!({ "banned": false }),
            )
            .await?
        }
        UserCmd::AddRole { id, role_id } => {
            c.post(&format!("/api/admin/users/{id}/roles/{role_id}"), json!({}))
                .await?
        }
        UserCmd::RemoveRole { id, role_id } => {
            c.delete(&format!("/api/admin/users/{id}/roles/{role_id}"))
                .await?
        }
        UserCmd::AddPerm { id, permission } => {
            c.post(
                &format!("/api/admin/users/{id}/permissions"),
                json!({ "permission": permission }),
            )
            .await?
        }
        UserCmd::RemovePerm { id, permission } => {
            c.delete(&format!(
                "/api/admin/users/{id}/permissions/{}",
                urlencode(&permission)
            ))
            .await?
        }
        UserCmd::SetCape { id, cape_id } => {
            c.put(
                &format!("/api/admin/users/{id}/cape"),
                json!({ "cape_id": cape_id }),
            )
            .await?
        }
    };
    print_json(&v);
    Ok(())
}
