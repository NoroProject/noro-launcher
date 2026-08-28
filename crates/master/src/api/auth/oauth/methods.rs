//! What you can sign in with on this instance. Public, so the login page never
//! shows a button with nothing behind it.

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
        // The flag alone isn't enough: WebAuthn doesn't initialise without
        // public URLs configured, and the button would lead to a refusal.
        "passkey": state.webauthn.is_some() && passkey_enabled(&state).await?,
    })))
}

/// Separate from whether WebAuthn is configured: the operator can switch the
/// method off even when it would technically work.
pub async fn passkey_enabled(state: &AppState) -> AppResult<bool> {
    Ok(crate::db::auth_methods::is_enabled(&state.db, PASSKEY).await?)
}

/// Called from every passkey handler: a disabled sign-in method has to be
/// disabled for direct API callers too, not just for the button on the site.
pub async fn require_passkey_enabled(state: &AppState) -> AppResult<()> {
    if passkey_enabled(state).await? {
        return Ok(());
    }
    Err(crate::error::AppError::BadRequest(
        "passkey sign-in is turned off on this instance".into(),
    ))
}
