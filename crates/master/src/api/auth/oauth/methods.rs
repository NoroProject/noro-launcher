//! Чем можно войти на этом инстансе. Публичная ручка: страница входа не должна
//! показывать кнопку, за которой ничего нет.

use super::config;
use super::provider::Provider;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

pub const PASSKEY: &str = "passkey";

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let providers: Vec<Value> = config::enabled(&state.db)
        .await?
        .into_iter()
        .map(|p: Provider| json!({ "provider": p.slug(), "name": p.display_name() }))
        .collect();

    Ok(Json(json!({
        "providers": providers,
        // Passkey требует не только флага: без публичных адресов WebAuthn не
        // инициализируется, и кнопка вела бы в отказ.
        "passkey": state.webauthn.is_some() && passkey_enabled(&state).await?,
    })))
}

/// Включён ли вход по passkey. Отдельно от наличия WebAuthn: оператор может
/// выключить способ, даже когда технически он доступен.
pub async fn passkey_enabled(state: &AppState) -> AppResult<bool> {
    Ok(crate::db::auth_methods::is_enabled(&state.db, PASSKEY).await?)
}

/// Отказ, если passkey выключен оператором. Проверяется в каждой ручке
/// passkey: выключенный способ входа обязан быть выключен и для того, кто
/// зовёт API напрямую, а не только для кнопки на сайте.
pub async fn require_passkey_enabled(state: &AppState) -> AppResult<()> {
    if passkey_enabled(state).await? {
        return Ok(());
    }
    Err(crate::error::AppError::BadRequest(
        "passkey sign-in is turned off on this instance".into(),
    ))
}
