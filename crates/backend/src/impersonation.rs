//! Impersonation, launcher side: trading a confirmed grant for a session.
//!
//! The token comes back over HTTPS in reply to the launcher's own request. Not
//! through the browser, not in a URL, and not as a process argument that shows
//! up in `ps`.

use anyhow::{bail, Result};
use uuid::Uuid;

pub struct Claimed {
    pub access_token: String,
    pub username: String,
}

pub async fn claim(
    http: &reqwest::Client,
    master_url: &str,
    access_token: &str,
    grant_id: Uuid,
) -> Result<Claimed> {
    let url = format!(
        "{}/api/launcher/impersonate/claim",
        master_url.trim_end_matches('/')
    );
    let resp = http
        .post(&url)
        .bearer_auth(access_token)
        .json(&serde_json::json!({ "grant_id": grant_id }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        bail!(
            "master refused the grant ({status}): {}",
            resp.text().await.unwrap_or_default()
        );
    }

    let value: serde_json::Value = resp.json().await?;
    let token = value
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("master returned no access token"))?;
    let username = value
        .get("user")
        .and_then(|u| u.get("username"))
        .and_then(|v| v.as_str())
        .unwrap_or("?");

    Ok(Claimed {
        access_token: token.to_string(),
        username: username.to_string(),
    })
}
