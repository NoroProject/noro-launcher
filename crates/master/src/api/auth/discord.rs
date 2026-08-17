//! Discord OAuth2: для сайта и для лаунчера (callback на localhost).

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use chrono::Duration;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

const DISCORD_AUTHORIZE: &str = "https://discord.com/api/oauth2/authorize";
const DISCORD_TOKEN: &str = "https://discord.com/api/oauth2/token";
const DISCORD_ME: &str = "https://discord.com/api/users/@me";

fn random_state() -> String {
    use rand::Rng;
    let bytes: [u8; 24] = rand::thread_rng().gen();
    hex::encode(bytes)
}

#[derive(Deserialize)]
pub struct LoginQuery {
    /// Куда вернуть пользователя после логина (для сайта).
    pub redirect: Option<String>,
}

/// Старт OAuth для сайта.
pub async fn login(
    State(state): State<AppState>,
    Query(q): Query<LoginQuery>,
) -> AppResult<Redirect> {
    let csrf = random_state();
    sqlx::query("INSERT INTO oauth_states (state, redirect) VALUES ($1, $2)")
        .bind(&csrf)
        .bind(&q.redirect)
        .execute(&state.db)
        .await?;
    Ok(Redirect::to(&authorize_url(
        &state,
        &csrf,
        &state.config.discord_redirect_uri(),
    )))
}

#[derive(Deserialize)]
pub struct LauncherLoginQuery {
    /// Порт локального HTTP-сервера лаунчера.
    pub port: u16,
}

/// Старт OAuth для лаунчера.
pub async fn launcher_login(
    State(state): State<AppState>,
    Query(q): Query<LauncherLoginQuery>,
) -> AppResult<Redirect> {
    let csrf = random_state();
    sqlx::query("INSERT INTO oauth_states (state, launcher_port) VALUES ($1, $2)")
        .bind(&csrf)
        .bind(q.port as i32)
        .execute(&state.db)
        .await?;
    Ok(Redirect::to(&authorize_url(
        &state,
        &csrf,
        &state.config.discord_launcher_redirect_uri(),
    )))
}

fn authorize_url(state: &AppState, csrf: &str, redirect_uri: &str) -> String {
    format!(
        "{DISCORD_AUTHORIZE}?client_id={}&redirect_uri={}&response_type=code&scope=identify&state={}",
        urlencoding(&state.config.discord_client_id),
        urlencoding(redirect_uri),
        csrf
    )
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(sqlx::FromRow)]
struct StateRow {
    redirect: Option<String>,
    launcher_port: Option<i32>,
}

/// Callback для сайта.
pub async fn callback(
    State(state): State<AppState>,
    Query(q): Query<CallbackQuery>,
) -> AppResult<Redirect> {
    let st = consume_state(&state, &q.state).await?;
    let (_user_id, session) = exchange_and_session(
        &state,
        &q.code,
        &state.config.discord_redirect_uri(),
        "site",
        Duration::days(7),
    )
    .await?;

    // Возврат на сайт с токенами в query.
    let base = st
        .redirect
        .unwrap_or_else(|| format!("{}/login/callback", state.config.public_url));
    let sep = if base.contains('?') { '&' } else { '?' };
    let target = format!(
        "{base}{sep}access_token={}&refresh_token={}",
        session.access_token, session.refresh_token
    );
    Ok(Redirect::to(&target))
}

/// Callback для лаунчера: отправляет токен на localhost:port.
pub async fn launcher_callback(
    State(state): State<AppState>,
    Query(q): Query<CallbackQuery>,
) -> AppResult<impl IntoResponse> {
    let st = consume_state(&state, &q.state).await?;
    let port = st
        .launcher_port
        .ok_or_else(|| AppError::BadRequest("state has no launcher_port".into()))?;

    let (user_id, session) = exchange_and_session(
        &state,
        &q.code,
        &state.config.discord_launcher_redirect_uri(),
        "launcher",
        Duration::days(30),
    )
    .await?;
    // Редирект отправляет на loopback БРАУЗЕР игрока. Раньше мастер сам делал
    // POST на 127.0.0.1 — свой собственный, а не игрока, поэтому с боевого
    // сервера токены не доходили никуда.
    let code = crate::db::create_launcher_code(&state.db, user_id, &session).await?;
    Ok(Redirect::to(&format!(
        "http://127.0.0.1:{port}/callback?code={code}"
    )))
}

#[derive(Deserialize)]
pub struct ExchangeReq {
    pub code: Uuid,
}

/// Обменять одноразовый код на токены сессии. Ходит сюда сам лаунчер по HTTPS.
pub async fn launcher_exchange(
    State(state): State<AppState>,
    Json(req): Json<ExchangeReq>,
) -> AppResult<Json<serde_json::Value>> {
    let (user_id, access_token, refresh_token) = crate::db::take_launcher_code(&state.db, req.code)
        .await?
        .ok_or_else(|| AppError::BadRequest("the sign-in code is invalid or expired".into()))?;
    let profile = crate::db::load_profile(&state.db, user_id).await?;
    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "user": profile,
    })))
}

async fn consume_state(state: &AppState, csrf: &str) -> AppResult<StateRow> {
    let row = sqlx::query_as::<_, StateRow>(
        "DELETE FROM oauth_states WHERE state = $1 RETURNING redirect, launcher_port",
    )
    .bind(csrf)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("invalid or expired state".into()))?;
    Ok(row)
}

/// Обменять code на токен Discord, получить профиль, создать/обновить пользователя и сессию.
async fn exchange_and_session(
    state: &AppState,
    code: &str,
    redirect_uri: &str,
    scope: &str,
    ttl: Duration,
) -> AppResult<(uuid::Uuid, crate::db::NewSession)> {
    let token_resp: Value = state
        .http()
        .post(DISCORD_TOKEN)
        .form(&[
            ("client_id", state.config.discord_client_id.as_str()),
            ("client_secret", state.config.discord_client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .json()
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    let access = token_resp["access_token"].as_str().ok_or_else(|| {
        AppError::Unauthorized(format!("Discord rejected the code: {token_resp}"))
    })?;

    let me: Value = state
        .http()
        .get(DISCORD_ME)
        .bearer_auth(access)
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .json()
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    let discord_id = me["id"]
        .as_str()
        .ok_or_else(|| AppError::Unauthorized("missing Discord id".into()))?;
    // Ник не подставляем: аккаунт создаётся один раз, и «player» остался бы с
    // игроком навсегда — вместе с чужими такими же «player».
    let username = me["username"]
        .as_str()
        .ok_or_else(|| AppError::Unauthorized("Discord returned no username".into()))?;
    let avatar = me["avatar"]
        .as_str()
        .map(|hash| format!("https://cdn.discordapp.com/avatars/{discord_id}/{hash}.png"));

    let (user_id, _is_new) =
        crate::db::find_or_create_user(&state.db, discord_id, username, avatar.as_deref()).await?;
    let session = crate::db::create_session(&state.db, user_id, scope, ttl).await?;
    crate::audit::record_by_user(
        state,
        user_id,
        crate::audit::actions::AUTH_LOGIN,
        serde_json::json!({ "method": "discord", "scope": scope }),
    )
    .await;
    Ok((user_id, session))
}

#[derive(Deserialize)]
pub struct RefreshReq {
    pub refresh_token: String,
}

/// Обновить access-токен по refresh-токену.
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshReq>,
) -> AppResult<Json<Value>> {
    let rt = uuid::Uuid::parse_str(&req.refresh_token)
        .map_err(|_| AppError::Unauthorized("invalid refresh token".into()))?;
    let session = crate::db::refresh_session(&state.db, rt, Duration::days(30))
        .await?
        .ok_or_else(|| AppError::Unauthorized("refresh token not found".into()))?;
    Ok(Json(serde_json::json!({
        "access_token": session.access_token,
        "refresh_token": session.refresh_token,
        "expires_at": session.expires_at,
    })))
}

/// Выйти: удалить текущую сессию (Bearer access-токен).
pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> AppResult<Json<Value>> {
    if let Some(tok) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|v| uuid::Uuid::parse_str(v.trim()).ok())
    {
        crate::db::delete_session(&state.db, tok).await?;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Минимальный URL-энкодер для query-параметров.
fn urlencoding(s: &str) -> String {
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
