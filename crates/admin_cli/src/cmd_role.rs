//! Role management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;
use serde_json::json;

#[derive(Subcommand)]
pub enum RoleCmd {
    /// List all roles.
    List,
    /// Create a role.
    Create {
        name: String,
        display_name: String,
        #[arg(long)]
        color: Option<String>,
    },
    /// Update a role.
    Edit {
        id: String,
        display_name: String,
        #[arg(long)]
        color: Option<String>,
        #[arg(long, default_value_t = false)]
        is_default: bool,
        #[arg(long, default_value_t = 0)]
        sort_order: i32,
    },
    /// Delete a role.
    Delete { id: String },
    /// Grant permission to role.
    GrantPerm { id: String, permission: String },
    /// Revoke permission from role.
    RevokePerm { id: String, permission: String },
}

pub async fn run(c: &Client, cmd: RoleCmd) -> Result<()> {
    let v = match cmd {
        RoleCmd::List => c.get("/api/admin/roles").await?,
        RoleCmd::Create {
            name,
            display_name,
            color,
        } => {
            c.post(
                "/api/admin/roles",
                json!({ "name": name, "display_name": display_name, "color": color }),
            )
            .await?
        }
        RoleCmd::Edit {
            id,
            display_name,
            color,
            is_default,
            sort_order,
        } => {
            c.put(
                &format!("/api/admin/roles/{id}"),
                json!({
                    "display_name": display_name, "color": color,
                    "is_default": is_default, "sort_order": sort_order
                }),
            )
            .await?
        }
        RoleCmd::Delete { id } => c.delete(&format!("/api/admin/roles/{id}")).await?,
        RoleCmd::GrantPerm { id, permission } => {
            c.post(
                &format!("/api/admin/roles/{id}/permissions"),
                json!({ "permission": permission }),
            )
            .await?
        }
        RoleCmd::RevokePerm { id, permission } => {
            c.delete(&format!(
                "/api/admin/roles/{id}/permissions/{}",
                urlencode(&permission)
            ))
            .await?
        }
    };
    print_json(&v);
    Ok(())
}
