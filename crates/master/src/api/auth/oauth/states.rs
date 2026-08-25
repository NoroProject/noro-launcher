//! CSRF-state входа: то, что помнит мастер, пока игрок ходит на платформу.
//!
//! Строка одноразовая — читается через `DELETE ... RETURNING`, поэтому один и
//! тот же `state` нельзя предъявить дважды.

use super::provider::Provider;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct StateRow {
    pub redirect: Option<String>,
    pub provider: String,
    /// Не пусто — это привязка платформы к уже вошедшему игроку, а не вход.
    pub link_user_id: Option<Uuid>,
}

fn random() -> String {
    use rand::Rng;
    let bytes: [u8; 24] = rand::thread_rng().gen();
    hex::encode(bytes)
}

/// Завести state и вернуть его значение для адреса платформы.
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

/// Забрать state и погасить его.
pub async fn consume(state: &AppState, csrf: &str) -> AppResult<StateRow> {
    sqlx::query_as::<_, StateRow>(
        "DELETE FROM oauth_states WHERE state = $1 RETURNING redirect, provider, link_user_id",
    )
    .bind(csrf)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("invalid or expired state".into()))
}
