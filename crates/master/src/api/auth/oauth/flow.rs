//! Вход через внешнюю платформу.
//!
//! Ручки общие для всех платформ: `/auth/{provider}/login` и
//! `/auth/{provider}/callback`. Лаунчер сюда не ходит — он логинится через
//! сайт (`/oauth2/authorize`), поэтому и адрес возврата у провайдера ровно
//! один, вместо пары «сайт + лаунчер» на каждую платформу.

use super::config;
use super::provider::Provider;
use super::remote;
use super::states;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::response::Redirect;
use chrono::Duration;
use serde::Deserialize;
use uuid::Uuid;

pub fn parse_provider(slug: &str) -> AppResult<Provider> {
    Provider::from_slug(slug)
        .ok_or_else(|| AppError::NotFound(format!("unknown sign-in provider {slug}")))
}

#[derive(Deserialize)]
pub struct LoginQuery {
    /// Куда вернуть игрока после входа.
    pub redirect: Option<String>,
}

/// Старт входа: запоминаем CSRF-state и уводим на платформу.
pub async fn login(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<LoginQuery>,
) -> AppResult<Redirect> {
    let p = parse_provider(&slug)?;
    let creds = config::creds(&state.db, p).await?;
    let csrf = states::create(&state, p, q.redirect.as_deref(), None).await?;
    Ok(Redirect::to(&remote::authorize_url(
        p,
        &creds,
        &csrf,
        &remote::redirect_uri(&state, p),
    )))
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

/// Возврат с платформы: вход либо привязка — смотря с чего начинали.
pub async fn callback(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<CallbackQuery>,
) -> AppResult<Redirect> {
    let p = parse_provider(&slug)?;
    let st = states::consume(&state, &q.state).await?;
    // Начинали вход одной платформой, вернулись другой — так state от чужого
    // провайдера не подставить.
    if st.provider != p.slug() {
        return Err(AppError::BadRequest(
            "state belongs to another provider".into(),
        ));
    }

    let creds = config::creds(&state.db, p).await?;
    let identity =
        remote::fetch_identity(&state, p, &creds, &q.code, &remote::redirect_uri(&state, p))
            .await?;

    if let Some(user_id) = st.link_user_id {
        return super::link::finish(&state, user_id, p, &identity, st.redirect).await;
    }

    let (_user_id, session) = sign_in(&state, p, &identity, "site", Duration::days(7)).await?;
    let base = st
        .redirect
        .unwrap_or_else(|| format!("{}/login/callback", state.config.public_url));
    let sep = if base.contains('?') { '&' } else { '?' };
    Ok(Redirect::to(&format!(
        "{base}{sep}access_token={}&refresh_token={}",
        session.access_token, session.refresh_token
    )))
}

/// Найти игрока по привязке (или завести нового) и открыть ему сессию.
pub async fn sign_in(
    state: &AppState,
    p: Provider,
    identity: &super::provider::RemoteIdentity,
    scope: &str,
    ttl: Duration,
) -> AppResult<(Uuid, crate::db::NewSession)> {
    let (user_id, _is_new) = crate::db::find_or_create_user(
        &state.db,
        p.slug(),
        &identity.id,
        &identity.username,
        identity.avatar.as_deref(),
    )
    .await?;
    let session = crate::db::create_session(&state.db, user_id, scope, ttl).await?;
    crate::audit::record_by_user(
        state,
        user_id,
        crate::audit::actions::AUTH_LOGIN,
        serde_json::json!({ "method": p.slug(), "scope": scope }),
    )
    .await;
    Ok((user_id, session))
}
