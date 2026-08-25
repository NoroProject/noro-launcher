//! Telegram authentication handlers with HMAC-SHA256 signature verification.

use std::collections::BTreeMap;

use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::Json;
use chrono::Duration;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::api::auth::oauth::config;
use crate::api::auth::oauth::provider::{Provider, RemoteIdentity};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TelegramAuthReq {
    pub init_data: Option<String>,
    pub id: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: Option<String>,
    pub hash: Option<String>,
}

#[derive(Serialize)]
pub struct TelegramAuthResp {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
}

pub async fn telegram_login(
    State(state): State<AppState>,
    Json(req): Json<TelegramAuthReq>,
) -> AppResult<Json<TelegramAuthResp>> {
    let creds = config::creds(&state.db, Provider::Telegram).await?;
    let bot_token = &creds.client_secret;

    let (identity, _user_id) = process_telegram_auth(&req, bot_token)?;
    let (user_id, session) = crate::api::auth::oauth::flow::sign_in(
        &state,
        Provider::Telegram,
        &identity,
        "site",
        Duration::days(7),
    )
    .await?;

    Ok(Json(TelegramAuthResp {
        access_token: session.access_token.to_string(),
        refresh_token: session.refresh_token.to_string(),
        user_id,
    }))
}

pub async fn telegram_callback(
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> AppResult<Redirect> {
    let creds = config::creds(&state.db, Provider::Telegram).await?;
    let bot_token = &creds.client_secret;

    if !verify_widget_hash(&params, bot_token) {
        return Err(AppError::Unauthorized("Invalid Telegram auth hash".into()));
    }

    let id = params
        .get("id")
        .cloned()
        .ok_or_else(|| AppError::BadRequest("Missing id".into()))?;
    let username = params
        .get("username")
        .cloned()
        .or_else(|| params.get("first_name").cloned())
        .unwrap_or_else(|| format!("tg_{id}"));
    let avatar = params.get("photo_url").cloned();

    let identity = RemoteIdentity {
        id,
        username,
        avatar,
    };
    let (_user_id, session) = crate::api::auth::oauth::flow::sign_in(
        &state,
        Provider::Telegram,
        &identity,
        "site",
        Duration::days(7),
    )
    .await?;

    let base = format!("{}/login/callback", state.config.public_url);
    let sep = if base.contains('?') { '&' } else { '?' };
    Ok(Redirect::to(&format!(
        "{base}{sep}access_token={}&refresh_token={}",
        session.access_token, session.refresh_token
    )))
}

fn process_telegram_auth(
    req: &TelegramAuthReq,
    bot_token: &str,
) -> AppResult<(RemoteIdentity, String)> {
    if let Some(ref init_data) = req.init_data {
        if let Some(map) = verify_init_data(init_data, bot_token) {
            if let Some(user_json) = map.get("user") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(user_json) {
                    let id = val["id"].to_string();
                    let username = val
                        .get("username")
                        .and_then(|s| s.as_str())
                        .or_else(|| val.get("first_name").and_then(|s| s.as_str()))
                        .unwrap_or("TelegramUser")
                        .to_string();
                    let avatar = val
                        .get("photo_url")
                        .and_then(|s| s.as_str())
                        .map(String::from);
                    return Ok((
                        RemoteIdentity {
                            id: id.clone(),
                            username,
                            avatar,
                        },
                        id,
                    ));
                }
            }
        }
    }

    let mut map = BTreeMap::new();
    if let Some(ref v) = req.id {
        map.insert("id".into(), v.clone());
    }
    if let Some(ref v) = req.first_name {
        map.insert("first_name".into(), v.clone());
    }
    if let Some(ref v) = req.last_name {
        map.insert("last_name".into(), v.clone());
    }
    if let Some(ref v) = req.username {
        map.insert("username".into(), v.clone());
    }
    if let Some(ref v) = req.photo_url {
        map.insert("photo_url".into(), v.clone());
    }
    if let Some(ref v) = req.auth_date {
        map.insert("auth_date".into(), v.clone());
    }
    if let Some(ref v) = req.hash {
        map.insert("hash".into(), v.clone());
    }

    if verify_widget_hash(&map, bot_token) {
        let id = req
            .id
            .clone()
            .ok_or_else(|| AppError::BadRequest("Missing id".into()))?;
        let username = req
            .username
            .clone()
            .or_else(|| req.first_name.clone())
            .unwrap_or_else(|| format!("tg_{id}"));
        let avatar = req.photo_url.clone();
        Ok((
            RemoteIdentity {
                id: id.clone(),
                username,
                avatar,
            },
            id,
        ))
    } else {
        Err(AppError::Unauthorized(
            "Invalid Telegram auth hash signature".into(),
        ))
    }
}

pub fn verify_widget_hash(params: &BTreeMap<String, String>, bot_token: &str) -> bool {
    let Some(hash) = params.get("hash") else {
        return false;
    };
    if let Some(auth_date_str) = params.get("auth_date") {
        if let Ok(auth_date) = auth_date_str.parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            if (now - auth_date).abs() > 86400 {
                return false;
            }
        }
    }

    let check_string = params
        .iter()
        .filter(|(k, _)| k.as_str() != "hash")
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    let secret_key = Sha256::digest(bot_token.as_bytes());
    let Ok(mut mac) = HmacSha256::new_from_slice(&secret_key) else {
        return false;
    };
    mac.update(check_string.as_bytes());
    let calculated_hash = hex::encode(mac.finalize().into_bytes());
    calculated_hash.eq_ignore_ascii_case(hash)
}

pub fn verify_init_data(init_data: &str, bot_token: &str) -> Option<BTreeMap<String, String>> {
    let mut params = BTreeMap::new();
    for pair in init_data.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            let decoded_v = urlencoding::decode(v)
                .unwrap_or(std::borrow::Cow::Borrowed(v))
                .to_string();
            params.insert(k.to_string(), decoded_v);
        }
    }

    let hash = params.get("hash")?.clone();
    if let Some(auth_date_str) = params.get("auth_date") {
        if let Ok(auth_date) = auth_date_str.parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            if (now - auth_date).abs() > 86400 {
                return None;
            }
        }
    }

    let check_string = params
        .iter()
        .filter(|(k, _)| k.as_str() != "hash")
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    let Ok(mut secret_mac) = HmacSha256::new_from_slice(b"WebAppData") else {
        return None;
    };
    secret_mac.update(bot_token.as_bytes());
    let secret_key = secret_mac.finalize().into_bytes();

    let Ok(mut mac) = HmacSha256::new_from_slice(&secret_key) else {
        return None;
    };
    mac.update(check_string.as_bytes());
    let calculated_hash = hex::encode(mac.finalize().into_bytes());
    if calculated_hash.eq_ignore_ascii_case(&hash) {
        Some(params)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_widget_hash_verification() {
        let mut map = BTreeMap::new();
        map.insert("id".into(), "123456789".into());
        map.insert("first_name".into(), "Alex".into());
        map.insert("username".into(), "alex_dev".into());
        map.insert(
            "auth_date".into(),
            chrono::Utc::now().timestamp().to_string(),
        );

        let bot_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11";
        let check_string = map
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n");
        let secret_key = Sha256::digest(bot_token.as_bytes());
        let mut mac = HmacSha256::new_from_slice(&secret_key).unwrap();
        mac.update(check_string.as_bytes());
        let hash = hex::encode(mac.finalize().into_bytes());

        map.insert("hash".into(), hash);
        assert!(verify_widget_hash(&map, bot_token));
    }
}
