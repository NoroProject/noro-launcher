//! `/api/setup/*` — то, что доступно до завершения первичной настройки.

use super::SetupAuth;
use crate::config::keys;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use ed25519_dalek::SigningKey;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Состояние установки. Единственная ручка без токена: без неё сайт не знает,
/// показывать визард или обычный вход.
pub async fn status(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let st = crate::db::instance_state(&state.db).await?;
    Ok(Json(json!({
        "setup_completed": st.setup_completed,
        // Секреты не показываются никогда, даже маской длины — только факт.
        "secrets": secrets_present(),
        // WebAuthn не работает по http:// на не-localhost адресе. Визард обязан
        // это заметить и предупредить, иначе оператор привяжет passkey, который
        // никогда не сработает.
        "secure_context": secure_context(&state),
    })))
}

/// Какие секреты видны в окружении. Значения не отдаются.
fn secrets_present() -> BTreeMap<&'static str, bool> {
    keys::SECRET_ENV
        .iter()
        .map(|k| (*k, crate::config::env_opt(k).is_some()))
        .collect()
}

/// Можно ли на этом адресе привязать passkey.
fn secure_context(state: &AppState) -> bool {
    let url = &state.config.web_url;
    url.starts_with("https://")
        || url.starts_with("http://localhost")
        || url.starts_with("http://127.0.0.1")
}

#[derive(Deserialize)]
pub struct SaveReq {
    /// Ключи из [`keys::ALL`]; всё остальное игнорируется.
    pub settings: BTreeMap<String, String>,
}

/// Сохранить настройки фазы A.
pub async fn save(
    State(state): State<AppState>,
    _auth: SetupAuth,
    Json(req): Json<SaveReq>,
) -> AppResult<Json<Value>> {
    let mut values: BTreeMap<String, Value> = BTreeMap::new();
    for (key, value) in req.settings {
        let known = keys::ALL.iter().any(|k| k.name == key);
        if !known {
            return Err(AppError::BadRequest(format!("unknown setting: {key}")));
        }
        values.insert(key, Value::String(value.trim().to_string()));
    }
    crate::db::set_settings(&state.db, &values, None).await?;
    Ok(Json(json!({ "ok": true, "saved": values.len() })))
}

#[derive(Deserialize)]
pub struct SignInReq {
    /// "discord" | "twitch" | "google".
    pub method: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}

/// Настроить вход через платформу на этапе визарда.
///
/// Отдельно от `save`: ключи приложения живут не в `instance_settings`, а в
/// `auth_methods` — там же, где потом их правит админка.
pub async fn save_sign_in(
    State(state): State<AppState>,
    _auth: SetupAuth,
    Json(req): Json<SignInReq>,
) -> AppResult<Json<Value>> {
    let p = crate::api::auth::oauth::Provider::from_slug(&req.method)
        .ok_or_else(|| AppError::BadRequest(format!("unknown sign-in method: {}", req.method)))?;

    let secret = req
        .client_secret
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let client_id = req.client_id.trim();
    // Платформа без ключей включённой быть не может: кнопка на странице входа
    // упиралась бы в отказ.
    let enabled = !client_id.is_empty();

    crate::db::auth_methods::save(&state.db, p.slug(), client_id, secret, enabled).await?;
    Ok(Json(json!({ "ok": true, "enabled": enabled })))
}

#[derive(Serialize)]
pub struct SigningKeyRes {
    /// Приватный seed. Показывается ОДИН раз и нигде не сохраняется — ни в БД,
    /// ни в логе. Не скопировал — генерируй заново.
    pub private_hex: String,
    pub public_hex: String,
    /// Готовая строка для `.env`.
    pub env_line: String,
}

/// Сгенерировать ключ подписи манифестов.
pub async fn generate_signing_key(_auth: SetupAuth) -> AppResult<Json<SigningKeyRes>> {
    let mut seed = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);
    let key = SigningKey::from_bytes(&seed);
    let private_hex = hex::encode(seed);

    Ok(Json(SigningKeyRes {
        env_line: format!("NORO_SIGNING_KEY={private_hex}"),
        public_hex: hex::encode(key.verifying_key().to_bytes()),
        private_hex,
    }))
}

/// Блок `.env` целиком — чтобы оператор скопировал его одним куском.
pub async fn env_block(State(state): State<AppState>, _auth: SetupAuth) -> AppResult<Json<Value>> {
    let stored = crate::db::all_settings(&state.db).await?;
    let mut lines = vec![
        "# Секреты и то, что нужно мастеру до подъёма БД.".to_string(),
        format!("DATABASE_URL={}", state.config.database_url),
        format!("NORO_BIND={}", state.config.bind_addr),
    ];
    for key in keys::SECRET_ENV {
        let present = crate::config::env_opt(key).is_some();
        lines.push(format!(
            "{key}={}",
            if present { "# уже задан" } else { "" }
        ));
    }
    lines.push(String::new());
    lines.push("# Остальное лежит в БД и правится в админке:".to_string());
    for (k, v) in &stored {
        lines.push(format!("#   {k} = {v}"));
    }
    Ok(Json(json!({ "env": lines.join("\n") })))
}

#[derive(Deserialize)]
pub struct RootReq {
    pub username: String,
}

/// Завести root-аккаунт и выдать recovery-коды.
///
/// Коды показываются один раз — это и есть вход, пока не настроен домен с TLS
/// и не привязан passkey.
pub async fn create_root(
    State(state): State<AppState>,
    _auth: SetupAuth,
    Json(req): Json<RootReq>,
) -> AppResult<Json<Value>> {
    if crate::db::root_exists(&state.db).await? {
        return Err(AppError::Conflict("the root account already exists".into()));
    }
    let name = req.username.trim();
    if name.is_empty()
        || name.len() > 16
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::BadRequest(
            "username: up to 16 latin letters, digits and underscores".into(),
        ));
    }

    let user = crate::db::create_local_account(&state.db, name, true).await?;
    // Полный доступ: это единственный аккаунт на пустом инстансе, и раздавать
    // ему права по одному не из чего — ролей ещё нет.
    crate::db::add_user_permission(&state.db, user.id, schema::PERM_SUPERADMIN, None, None).await?;
    let codes = crate::db::issue_recovery_codes(&state.db, user.id).await?;

    tracing::info!(user = %user.id, "создан root-аккаунт");
    Ok(Json(json!({
        "id": user.id,
        "username": user.mc_username,
        "recovery_codes": codes,
    })))
}

/// Завершить настройку. Токен сжигается, файл удаляется.
pub async fn complete(State(state): State<AppState>, _auth: SetupAuth) -> AppResult<Json<Value>> {
    // Смотрим в БД, а не в `state.config`: конфиг читается при старте, а
    // настройки визард сохранил только что. Проверка по памяти требовала бы
    // рестарта между «сохранил» и «завершил» — там, где визард его не просит.
    let stored = crate::db::all_settings(&state.db).await?;
    let filled = |key: &str| {
        stored
            .get(key)
            .and_then(|v| v.as_str())
            .is_some_and(|v| !v.trim().is_empty())
    };
    if !filled(keys::PUBLIC_URL.name) || !filled(keys::WEB_URL.name) {
        return Err(AppError::BadRequest(
            "the public website and API addresses are not set".into(),
        ));
    }
    // Без операторского аккаунта завершённая настройка означала бы инстанс,
    // в который никто не может войти.
    if !crate::db::root_exists(&state.db).await? {
        return Err(AppError::BadRequest("no root account created yet".into()));
    }
    crate::db::complete_setup(&state.db).await?;
    super::token::burn_token_file(&state.config.data_dir).await;
    tracing::info!("первичная настройка завершена");
    Ok(Json(json!({ "ok": true, "restart_required": true })))
}
