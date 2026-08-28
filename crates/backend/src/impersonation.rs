//! Сторона лаунчера: подтверждение входа и обмен гранта на сессию.
//!
//! Токен приходит по HTTPS в ответ на запрос самого лаунчера — не через
//! браузер, не через URL и не аргументом процесса, который видно в `ps`.

use anyhow::{bail, Result};
use uuid::Uuid;

/// Что вернул мастер в обмен на грант.
pub struct Claimed {
    pub access_token: String,
    pub username: String,
}

/// Обменять подтверждённый грант на сессию игрока.
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
            "мастер отказал ({status}): {}",
            resp.text().await.unwrap_or_default()
        );
    }

    let value: serde_json::Value = resp.json().await?;
    let token = value
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("мастер не вернул токен"))?;
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
