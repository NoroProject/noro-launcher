//! News management commands.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use serde_json::json;

#[derive(Subcommand)]
pub enum NewsCmd {
    /// List all news.
    List,
    /// Create a news item.
    Create {
        title: String,
        body: String,
        #[arg(long)]
        pinned: bool,
    },
    /// Edit a news item.
    Edit {
        id: String,
        title: String,
        body: String,
        #[arg(long)]
        preview: Option<String>,
        #[arg(long)]
        pinned: bool,
    },
    /// Delete a news item.
    Delete { id: String },
    /// Upload a news image.
    UploadImage { file: String },
}

pub async fn run(c: &Client, cmd: NewsCmd) -> Result<()> {
    let v = match cmd {
        NewsCmd::List => c.get("/api/admin/news").await?,
        NewsCmd::Create {
            title,
            body,
            pinned,
        } => {
            c.post(
                "/api/admin/news",
                json!({ "title": title, "body": body, "pinned": pinned }),
            )
            .await?
        }
        NewsCmd::Edit {
            id,
            title,
            body,
            preview,
            pinned,
        } => {
            c.put(
                &format!("/api/admin/news/{id}"),
                json!({
                    "title": title, "body": body,
                    "preview_img_url": preview, "pinned": pinned
                }),
            )
            .await?
        }
        NewsCmd::Delete { id } => c.delete(&format!("/api/admin/news/{id}")).await?,
        NewsCmd::UploadImage { file } => {
            c.upload_image("/api/admin/news/image", &file, "image")
                .await?
        }
    };
    print_json(&v);
    Ok(())
}
