//! Отправка бандла мастеру по инициативе игрока.
//!
//! Ни запроса от админа, ни гранта, ни TTL: игрок сам нажал «Сообщить о
//! проблеме». Это покрывает большую часть случаев, ради которых нужен был бы
//! админский запрос, и ничего у игрока не спрашивает — он и есть инициатор.

use anyhow::{bail, Result};
use std::path::Path;
use uuid::Uuid;

/// То же, но по запросу админа: бандл привязывается к запросу и игрок его уже
/// не удалит — иначе принудительный режим не имел бы смысла.
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

/// Собрать, упаковать и отправить. Возвращает id бандла на мастере.
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
        bail!("нечего отправлять: логов ещё нет");
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
        bail!("мастер отказал ({status}): {body}");
    }

    let value: serde_json::Value = resp.json().await?;
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| anyhow::anyhow!("мастер не вернул id бандла"))?;
    Ok(id)
}
