//! Impersonation, launcher side: trading a confirmed grant for a session.
//!
//! The token comes back over HTTPS in reply to the launcher's own request. Not
//! through the browser, not in a URL, and not as a process argument that shows
//! up in `ps`.

use anyhow::{anyhow, Result};
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
    let api =
        crate::master_api::MasterApi::new(http.clone(), master_url, Some(access_token.to_string()))
            .ok_or_else(|| anyhow!("not signed in"))?;
    let value = api
        .claim_impersonation(grant_id)
        .await
        .map_err(|e| e.context("master refused the grant"))?;
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
