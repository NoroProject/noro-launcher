//! Собственный Yggdrasil-совместимый API для authlib-injector.
//!
//! Лаунчер передаёт MC-клиенту наш session access_token как --accessToken.
//! При входе на сервер клиент зовёт join (с этим токеном), а сервер — hasJoined.

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use base64::Engine;
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

fn undash(u: &Uuid) -> String {
    u.simple().to_string()
}

fn parse_uuid_loose(s: &str) -> Option<Uuid> {
    Uuid::parse_str(s).ok()
}

/// Корневой ALI-манифест authlib-injector.
pub async fn root(State(state): State<AppState>) -> Json<Value> {
    // signaturePublicKey в формате PEM из нашего ed25519? authlib-injector ждёт RSA.
    // Мы раздаём текстуры без подписи, поэтому ключ-заглушку не публикуем как RSA;
    // оставляем поле пустым — клиенты работают с unsigned-текстурами.
    Json(json!({
        "meta": {
            "serverName": "Noro",
            "implementationName": "noro-master",
            "implementationVersion": env!("CARGO_PKG_VERSION"),
            "feature.non_email_login": true,
            "links": {
                "homepage": state.config.public_url,
            }
        },
        "skinDomains": skin_domains(&state.config),
        "signaturePublicKey": ""
    }))
}

fn skin_domains(config: &crate::config::Config) -> Vec<String> {
    let mut domains = Vec::new();
    let mut add_url = |url: &str| {
        let host_and_port = url.split("://").nth(1).unwrap_or(url).split('/').next().unwrap_or(url);
        let pure_host = host_and_port.split(':').next().unwrap_or(host_and_port);
        if !pure_host.is_empty() && !domains.contains(&pure_host.to_string()) {
            domains.push(pure_host.to_string());
        }
        if host_and_port != pure_host && !domains.contains(&host_and_port.to_string()) {
            domains.push(host_and_port.to_string());
        }
    };

    add_url(&config.public_url);
    if let Some(cdn) = &config.files_cdn_url {
        add_url(cdn);
    }
    if let Some(s3) = &config.s3 {
        add_url(&s3.public_url);
    }
    if !domains.iter().any(|d| d == ".noro.gg") {
        domains.push(".noro.gg".to_string());
    }
    domains
}

// --- authserver (legacy, для совместимости) ---

#[derive(Deserialize)]
pub struct AuthenticateReq {
    #[serde(rename = "clientToken")]
    pub client_token: Option<String>,
    // username/password игнорируются — авторизация через Discord/лаунчер.
}

/// Заглушка authenticate: возвращает ошибку, т.к. вход только через лаунчер.
pub async fn authenticate(Json(_req): Json<AuthenticateReq>) -> AppResult<Json<Value>> {
    Err(AppError::Forbidden(
        "вход только через лаунчер (Discord)".into(),
    ))
}

#[derive(Deserialize)]
pub struct RefreshReq {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "clientToken")]
    pub client_token: Option<String>,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshReq>,
) -> AppResult<Json<Value>> {
    let token = parse_uuid_loose(&req.access_token)
        .ok_or_else(|| AppError::Unauthorized("неверный токен".into()))?;
    let row = crate::db::user_by_access_token(&state.db, token)
        .await?
        .ok_or_else(|| AppError::Unauthorized("сессия истекла".into()))?;
    Ok(Json(json!({
        "accessToken": req.access_token,
        "clientToken": req.client_token.unwrap_or_default(),
        "selectedProfile": {
            "id": undash(&row.mc_uuid),
            "name": row.mc_username,
        }
    })))
}

pub async fn validate(State(state): State<AppState>, Json(req): Json<RefreshReq>) -> StatusCode {
    match parse_uuid_loose(&req.access_token) {
        Some(token) => match crate::db::user_by_access_token(&state.db, token).await {
            Ok(Some(_)) => StatusCode::NO_CONTENT,
            _ => StatusCode::FORBIDDEN,
        },
        None => StatusCode::FORBIDDEN,
    }
}

pub async fn invalidate(State(state): State<AppState>, Json(req): Json<RefreshReq>) -> StatusCode {
    if let Some(token) = parse_uuid_loose(&req.access_token) {
        let _ = crate::db::delete_session(&state.db, token).await;
    }
    StatusCode::NO_CONTENT
}

// --- session: join / hasJoined ---

#[derive(Deserialize)]
pub struct JoinReq {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "selectedProfile")]
    pub selected_profile: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
}

/// Клиент сообщает серверу о намерении войти.
pub async fn join(
    State(state): State<AppState>,
    Json(req): Json<JoinReq>,
) -> AppResult<StatusCode> {
    let token = parse_uuid_loose(&req.access_token)
        .ok_or_else(|| AppError::Forbidden("неверный токен".into()))?;
    let row = crate::db::user_by_access_token(&state.db, token)
        .await?
        .ok_or_else(|| AppError::Forbidden("сессия истекла".into()))?;

    // Бан проверяется здесь, а не только в REST-миддлваре: Yggdrasil — это
    // отдельный путь входа, и без этой проверки забаненный игрок с ещё живой
    // сессией спокойно заходил на сервер.
    if row.banned {
        return Err(AppError::Forbidden("аккаунт заблокирован".into()));
    }

    // selectedProfile должен совпадать с MC UUID пользователя.
    let sel = req.selected_profile.replace('-', "");
    if undash(&row.mc_uuid) != sel {
        return Err(AppError::Forbidden("профиль не совпадает".into()));
    }

    let expires = Utc::now() + Duration::minutes(5);
    sqlx::query(
        "INSERT INTO mc_sessions (user_id, access_token, server_id, expires_at)
         VALUES ($1,$2,$3,$4)
         ON CONFLICT (user_id) DO UPDATE SET access_token=$2, server_id=$3, expires_at=$4",
    )
    .bind(row.id)
    .bind(token)
    .bind(&req.server_id)
    .bind(expires)
    .execute(&state.db)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct HasJoinedQuery {
    pub username: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
}

/// Сервер проверяет, что игрок действительно начал вход.
pub async fn has_joined(
    State(state): State<AppState>,
    Query(q): Query<HasJoinedQuery>,
) -> Result<Json<Value>, StatusCode> {
    let row = sqlx::query_as::<_, crate::db::models::UserRow>(
        // banned = FALSE и здесь: бан может прилететь между join и hasJoined,
        // а это последний рубеж перед тем, как сервер впустит игрока.
        "SELECT u.* FROM users u JOIN mc_sessions s ON s.user_id = u.id
         WHERE u.mc_username = $1 AND s.server_id = $2 AND s.expires_at > NOW()
           AND u.banned = FALSE",
    )
    .bind(&q.username)
    .bind(&q.server_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(u) => Ok(Json(profile_json(&state, &u))),
        None => Err(StatusCode::NO_CONTENT),
    }
}

/// Профиль по UUID (с текстурами).
pub async fn profile(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let parsed = parse_uuid_loose(&uuid).ok_or(StatusCode::BAD_REQUEST)?;
    let u =
        sqlx::query_as::<_, crate::db::models::UserRow>("SELECT * FROM users WHERE mc_uuid = $1")
            .bind(parsed)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NO_CONTENT)?;
    Ok(Json(profile_json(&state, &u)))
}

#[derive(Deserialize)]
pub struct BulkReq(pub Vec<String>);

/// Массовый поиск username → uuid.
pub async fn profiles_bulk(
    State(state): State<AppState>,
    Json(names): Json<Vec<String>>,
) -> Result<Json<Value>, StatusCode> {
    let mut out = Vec::new();
    for name in names.into_iter().take(100) {
        if let Ok(Some(u)) = sqlx::query_as::<_, crate::db::models::UserRow>(
            "SELECT * FROM users WHERE mc_username = $1",
        )
        .bind(&name)
        .fetch_optional(&state.db)
        .await
        {
            out.push(json!({ "id": undash(&u.mc_uuid), "name": u.mc_username }));
        }
    }
    Ok(Json(Value::Array(out)))
}

/// Сформировать профиль с base64-свойством textures.
fn profile_json(state: &AppState, u: &crate::db::models::UserRow) -> Value {
    let mut textures = serde_json::Map::new();
    // Свой скин либо общий Стив: пустой блок textures заставлял клиент
    // выбирать Стива или Алекса самостоятельно, и вид расходился с кабинетом.
    let skin = u
        .skin_url
        .clone()
        .unwrap_or_else(|| state.config.default_skin_url());
    textures.insert("SKIN".into(), json!({ "url": skin }));
    if let Some(cape) = &u.cape_url {
        textures.insert("CAPE".into(), json!({ "url": cape }));
    }

    let textures_payload = json!({
        "timestamp": Utc::now().timestamp_millis(),
        "profileId": undash(&u.mc_uuid),
        "profileName": u.mc_username,
        "textures": textures,
    });
    let b64 = base64::engine::general_purpose::STANDARD
        .encode(serde_json::to_vec(&textures_payload).unwrap_or_default());

    json!({
        "id": undash(&u.mc_uuid),
        "name": u.mc_username,
        "properties": [
            { "name": "textures", "value": b64 }
        ]
    })
}
