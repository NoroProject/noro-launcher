//! Build file operations: list, upload, download content, edit, move, delete.

use anyhow::Result;
use clap::Subcommand;

use crate::client::{print_json, Client};
use serde_json::json;

#[derive(Subcommand)]
pub enum FileCmd {
    /// List all files in a build.
    List { build_id: String },
    /// Upload a file to a build.
    Upload {
        build_id: String,
        file: String,
        #[arg(long)]
        path: Option<String>,
    },
    /// Download a build file to local path.
    Download {
        build_id: String,
        path: String,
        #[arg(long)]
        out: String,
    },
    /// Download a file by SHA1 hash to local path.
    DownloadSha1 {
        sha1: String,
        #[arg(long)]
        out: String,
    },
    /// Get text content of a file in a build.
    Content { build_id: String, path: String },
    /// Update text content of a file in a build.
    UpdateContent {
        build_id: String,
        path: String,
        content: String,
    },
    /// Delete a single file by ID.
    Delete { build_id: String, file_id: String },
    /// Delete all files matching a path prefix (folder).
    DeletePrefix { build_id: String, prefix: String },
    /// Move/rename a file or folder.
    Move {
        build_id: String,
        from: String,
        to: String,
    },
}

pub async fn run(c: &Client, cmd: FileCmd) -> Result<()> {
    match cmd {
        FileCmd::List { build_id } => {
            let v = c
                .get(&format!("/api/admin/builds/{build_id}/files"))
                .await?;
            print_json(&v);
        }
        FileCmd::Upload {
            build_id,
            file,
            path,
        } => {
            let v = c
                .upload(
                    &format!("/api/admin/builds/{build_id}/files"),
                    &file,
                    path.as_deref(),
                )
                .await?;
            print_json(&v);
        }
        FileCmd::Download {
            build_id,
            path,
            out,
        } => {
            let files = c
                .get(&format!("/api/admin/builds/{build_id}/files"))
                .await?;
            let arr = files
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("invalid file list"))?;
            let found = arr
                .iter()
                .find(|f| f["path"].as_str() == Some(&path))
                .ok_or_else(|| anyhow::anyhow!("file not found in build: {path}"))?;
            let sha1 = found["sha1"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("missing sha1"))?;
            c.download(&format!("/files/{sha1}"), &out).await?;
        }
        FileCmd::DownloadSha1 { sha1, out } => {
            c.download(&format!("/files/{sha1}"), &out).await?;
        }
        FileCmd::Content { build_id, path } => {
            let encoded = crate::util::urlencode(&path);
            let v = c
                .get(&format!(
                    "/api/admin/builds/{build_id}/files/content?path={encoded}"
                ))
                .await?;
            print_json(&v);
        }
        FileCmd::UpdateContent {
            build_id,
            path,
            content,
        } => {
            let v = c
                .put(
                    &format!("/api/admin/builds/{build_id}/files/content"),
                    json!({ "path": path, "content": content }),
                )
                .await?;
            print_json(&v);
        }
        FileCmd::Delete { build_id, file_id } => {
            let v = c
                .delete(&format!("/api/admin/builds/{build_id}/files/{file_id}"))
                .await?;
            print_json(&v);
        }
        FileCmd::DeletePrefix { build_id, prefix } => {
            let encoded = crate::util::urlencode(&prefix);
            let v = c
                .delete(&format!(
                    "/api/admin/builds/{build_id}/files/prefix?path={encoded}"
                ))
                .await?;
            print_json(&v);
        }
        FileCmd::Move { build_id, from, to } => {
            let v = c
                .post(
                    &format!("/api/admin/builds/{build_id}/files/move"),
                    json!({ "from": from, "to": to }),
                )
                .await?;
            print_json(&v);
        }
    };
    Ok(())
}
