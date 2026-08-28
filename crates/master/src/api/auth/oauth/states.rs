//! The CSRF state the master remembers while a player is off at the platform.
//!
//! Single use: it is read with `DELETE ... RETURNING`, so the same `state`
//! can't be presented twice.

use super::provider::Provider;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct StateRow {
    pub redirect: Option<String>,
    pub provider: String,
    /// Set when this is a platform link for an already signed-in player rather
    /// than a sign-in.
    pub link_user_id: Option<Uuid>,
}

fn random() -> String {
    use rand::Rng;
    let bytes: [u8; 24] = rand::thread_rng().gen();
    hex::encode(bytes)
}

/// Returns the state value to put in the platform's authorize URL.
pub async fn create(
    state: &AppState,
    p: Provider,
    redirect: Option<&str>,
    link_user_id: Option<Uuid>,
) -> AppResult<String> {
    let csrf = random();
    sqlx::query(
        "INSERT INTO oauth_states (state, redirect, provider, link_user_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(&csrf)
    .bind(redirect)
    .bind(p.slug())
    .bind(link_user_id)
    .execute(&state.db)
    .await?;
    Ok(csrf)
}

pub async fn consume(state: &AppState, csrf: &str) -> AppResult<StateRow> {
    sqlx::query_as::<_, StateRow>(
        "DELETE FROM oauth_states WHERE state = $1 RETURNING redirect, provider, link_user_id",
    )
    .bind(csrf)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("invalid or expired state".into()))
}
