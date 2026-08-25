//! OAuth2 Applications management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use crate::util::urlencode;

#[derive(Subcommand)]
pub enum OauthCmd {
    /// List OAuth2 applications.
    List,
    /// Create an OAuth2 application.
    Create {
        name: String,
        #[arg(long)]
        redirect_uri: String,
    },
    /// Delete an OAuth2 application.
    Delete { client_id: String },
}

pub async fn run(c: &Client, cmd: OauthCmd) -> Result<()> {
    match cmd {
        OauthCmd::List => {
            let v = c.get("/api/admin/oauth-apps").await?;
            print_json(&v);
        }
        OauthCmd::Create { name, redirect_uri } => {
            let body = serde_json::json!({ "name": name, "redirect_uri": redirect_uri });
            let v = c.post("/api/admin/oauth-apps", body).await?;
            print_json(&v);
        }
        OauthCmd::Delete { client_id } => {
            let path = format!("/api/admin/oauth-apps/{}", urlencode(&client_id));
            let v = c.delete(&path).await?;
            print_json(&v);
        }
    }
    Ok(())
}
