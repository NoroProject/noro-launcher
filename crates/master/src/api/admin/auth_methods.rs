//! Админ: способы входа — какие включены и с какими ключами приложения.
//!
//! Секрет наружу не отдаётся никогда, только факт «задан». Пустой секрет в
//! запросе означает «оставить прежний»: иначе сохранение формы, где поле
//! показано пустым, стирало бы рабочий ключ.

use super::settings::{PERM_SETTINGS_EDIT, PERM_SETTINGS_VIEW};
use crate::api::auth::oauth::provider::Provider;
use crate::api::auth::AdminAuth;
use crate::audit;
use crate::db::auth_methods::{self, AuthMethodRow};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

/// Способы, у которых нет ключей приложения, — только тумблер.
const TOGGLE_ONLY: &[&str] = &[crate::api::auth::oauth::methods::PASSKEY];

fn known(method: &str) -> bool {
    Provider::from_slug(method).is_some() || TOGGLE_ONLY.contains(&method)
}

pub async fn list(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Json<Value>> {
    admin.require(PERM_SETTINGS_VIEW)?;
    let rows = auth_methods::all(&state.db).await?;
    let row = |m: &str| rows.iter().find(|r| r.method == m).cloned();

    let mut items: Vec<Value> = Provider::ALL
        .into_iter()
        .map(|p| {
            let r = row(p.slug());
            let creds = crate::api::auth::oauth::config::resolve(p, r.as_ref());
            json!({
                "method": p.slug(),
                "name": p.display_name(),
                "kind": "oauth",
                "enabled": enabled_of(&r),
                "client_id": creds.client_id,
                "secret_set": !creds.client_secret.is_empty(),
                // Оператору это нужно скопировать в приложение на стороне
                // платформы — иначе она откажет с «redirect_uri mismatch».
                "redirect_uri": redirect_uri(&state, p),
                "configured": creds.is_set(),
                // Ключ из окружения правкой в админке не перекрыть — честнее
                // сказать об этом, чем молча не применять форму.
                "from_env": crate::config::env_opt(&format!("{}_CLIENT_ID", p.slug().to_uppercase()))
                    .is_some(),
            })
        })
        .collect();

    let passkey = row(crate::api::auth::oauth::methods::PASSKEY);
    items.push(json!({
        "method": crate::api::auth::oauth::methods::PASSKEY,
        "name": "Passkey",
        "kind": "passkey",
        "enabled": enabled_of(&passkey),
        // Без публичных адресов WebAuthn не поднимается вовсе.
        "configured": state.webauthn.is_some(),
    }));

    Ok(Json(json!({ "methods": items })))
}

fn enabled_of(row: &Option<AuthMethodRow>) -> bool {
    row.as_ref().is_none_or(|r| r.enabled)
}

fn redirect_uri(state: &AppState, p: Provider) -> String {
    format!("{}/auth/{}/callback", state.config.public_url, p.slug())
}

#[derive(Deserialize)]
pub struct SaveReq {
    #[serde(default)]
    pub client_id: String,
    /// Пусто или отсутствует — секрет не трогаем.
    pub client_secret: Option<String>,
    pub enabled: bool,
}

pub async fn save(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(method): Path<String>,
    Json(req): Json<SaveReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SETTINGS_EDIT)?;
    if !known(&method) {
        return Err(AppError::NotFound(format!(
            "unknown sign-in method {method}"
        )));
    }

    let secret = req
        .client_secret
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    auth_methods::save(
        &state.db,
        &method,
        req.client_id.trim(),
        secret,
        req.enabled,
    )
    .await?;

    audit::record(
        &state,
        &admin.actor,
        audit::actions::SETTINGS_UPDATE,
        None,
        json!({
            "auth_method": method,
            "enabled": req.enabled,
            "secret_changed": secret.is_some(),
        }),
    )
    .await;

    Ok(Json(json!({ "ok": true })))
}
