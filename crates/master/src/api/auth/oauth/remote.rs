//! Разговор с платформой: code → токен → профиль.

use super::config::Creds;
use super::provider::{Provider, RemoteIdentity};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde_json::Value;

/// URL, на который платформа вернёт игрока. Должен совпадать с тем, что
/// зарегистрирован в приложении на её стороне, — отсюда единый вид адреса.
/// Он один на платформу: лаунчер логинится через сайт и своего адреса не имеет.
pub fn redirect_uri(state: &AppState, p: Provider) -> String {
    format!("{}/auth/{}/callback", state.config.public_url, p.slug())
}

pub fn authorize_url(p: Provider, creds: &Creds, csrf: &str, redirect_uri: &str) -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        p.authorize_url(),
        urlencode(&creds.client_id),
        urlencode(redirect_uri),
        urlencode(p.scope()),
        csrf
    )
}

/// Обменять одноразовый code на профиль игрока у платформы.
pub async fn fetch_identity(
    state: &AppState,
    p: Provider,
    creds: &Creds,
    code: &str,
    redirect_uri: &str,
) -> AppResult<RemoteIdentity> {
    let mut req = state.http().post(p.token_url());
    if p == Provider::Telegram {
        use base64::Engine;
        let auth = base64::engine::general_purpose::STANDARD.encode(format!(
            "{}:{}",
            creds.client_id, creds.client_secret
        ));
        req = req.header("Authorization", format!("Basic {auth}"));
    }

    let res = req
        .form(&[
            ("client_id", creds.client_id.as_str()),
            ("client_secret", creds.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    let status = res.status();
    let text = res.text().await.map_err(|e| AppError::Other(e.into()))?;
    let token_resp: Value = serde_json::from_str(&text).map_err(|_| {
        AppError::Unauthorized(format!(
            "{} token endpoint returned non-JSON response ({status}): {text}",
            p.display_name()
        ))
    })?;

    let body = if p == Provider::Telegram {
        let id_token = token_resp["id_token"].as_str().ok_or_else(|| {
            AppError::Unauthorized(format!(
                "Telegram token response missing id_token: {token_resp}"
            ))
        })?;
        parse_jwt_payload(id_token).ok_or_else(|| {
            AppError::Unauthorized("Failed to parse Telegram id_token payload".into())
        })?
    } else {
        let access = token_resp["access_token"].as_str().ok_or_else(|| {
            AppError::Unauthorized(format!(
                "{} rejected the code: {token_resp}",
                p.display_name()
            ))
        })?;

        let u_url = p.userinfo_url().ok_or_else(|| {
            AppError::Unauthorized(format!("{} does not support userinfo endpoint", p.display_name()))
        })?;
        let mut u_req = state.http().get(u_url).bearer_auth(access);
        // Helix требует ещё и client_id: без заголовка ответ — 401 с пустым телом.
        if p == Provider::Twitch {
            u_req = u_req.header("Client-Id", creds.client_id.as_str());
        }
        let u_res = u_req
            .send()
            .await
            .map_err(|e| AppError::Other(e.into()))?;
        let u_status = u_res.status();
        let u_text = u_res.text().await.map_err(|e| AppError::Other(e.into()))?;
        serde_json::from_str(&u_text).map_err(|_| {
            AppError::Unauthorized(format!(
                "{} userinfo endpoint returned non-JSON response ({u_status}): {u_text}",
                p.display_name()
            ))
        })?
    };

    p.parse_identity(&body).ok_or_else(|| {
        AppError::Unauthorized(format!("{} returned no usable profile", p.display_name()))
    })
}

fn parse_jwt_payload(jwt: &str) -> Option<Value> {
    let part = jwt.split('.').nth(1)?;
    let mut s = part.replace('-', "+").replace('_', "/");
    while s.len() % 4 != 0 {
        s.push('=');
    }
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD.decode(s).ok()?;
    serde_json::from_slice(&decoded).ok()
}

/// Минимальный URL-энкодер для query-параметров.
pub fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
