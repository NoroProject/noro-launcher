//! Uploading a bundle to the master.
//!
//! The player-initiated path needs no admin request, grant or TTL — they pressed
//! the button, so there is nobody left to ask for consent.

use anyhow::{bail, Result};
use std::path::Path;
use uuid::Uuid;

/// The admin-requested path. Passing the `request_id` is what ties the bundle
/// to the request on the master, and a tied bundle can't be deleted by the
/// player.
pub async fn send_for_request(
    http: &reqwest::Client,
    master_url: &str,
    access_token: &str,
    instance_dir: &Path,
    server_id: Option<Uuid>,
    request_id: Uuid,
) -> Result<Uuid> {
    upload(
        http,
        master_url,
        access_token,
        instance_dir,
        server_id,
        "",
        Some(request_id),
    )
    .await
}

/// Returns the bundle's id on the master.
pub async fn send(
    http: &reqwest::Client,
    master_url: &str,
    access_token: &str,
    instance_dir: &Path,
    server_id: Option<Uuid>,
    note: &str,
) -> Result<Uuid> {
    upload(
        http,
        master_url,
        access_token,
        instance_dir,
        server_id,
        note,
        None,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn upload(
    http: &reqwest::Client,
    master_url: &str,
    access_token: &str,
    instance_dir: &Path,
    server_id: Option<Uuid>,
    note: &str,
    request_id: Option<Uuid>,
) -> Result<Uuid> {
    let bundle = super::collect(instance_dir, None, &[]).await;
    if bundle.files.is_empty() {
        bail!("nothing to send: no logs yet");
    }
    let archive = super::pack(&bundle)?;

    let mut url = format!(
        "{}/api/launcher/support-bundle?note={}",
        master_url.trim_end_matches('/'),
        urlencoding::encode(note)
    );
    if let Some(id) = server_id {
        url.push_str(&format!("&server_id={id}"));
    }
    if let Some(id) = request_id {
        url.push_str(&format!("&request_id={id}"));
    }

    let resp = http
        .post(&url)
        .bearer_auth(access_token)
        .header("content-type", "application/zip")
        .body(archive)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("master refused ({status}): {body}");
    }

    let value: serde_json::Value = resp.json().await?;
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| anyhow::anyhow!("master returned no bundle id"))?;
    Ok(id)
}
