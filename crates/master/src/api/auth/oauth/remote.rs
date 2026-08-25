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
    let res = state
        .http()
        .post(p.token_url())
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

    let access = token_resp["access_token"].as_str().ok_or_else(|| {
        AppError::Unauthorized(format!(
            "{} rejected the code: {token_resp}",
            p.display_name()
        ))
    })?;

    let mut req = state.http().get(p.userinfo_url()).bearer_auth(access);
    // Helix требует ещё и client_id: без заголовка ответ — 401 с пустым телом.
    if p == Provider::Twitch {
        req = req.header("Client-Id", creds.client_id.as_str());
    }
    let u_res = req
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    let u_status = u_res.status();
    let u_text = u_res.text().await.map_err(|e| AppError::Other(e.into()))?;
    let body: Value = serde_json::from_str(&u_text).map_err(|_| {
        AppError::Unauthorized(format!(
            "{} userinfo endpoint returned non-JSON response ({u_status}): {u_text}",
            p.display_name()
        ))
    })?;

    p.parse_identity(&body).ok_or_else(|| {
        AppError::Unauthorized(format!("{} returned no usable profile", p.display_name()))
    })
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
