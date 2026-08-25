//! Привязка второй, третьей и так далее платформы к уже существующему аккаунту.

use super::flow::parse_provider;
use super::provider::{Provider, RemoteIdentity};
use super::{config, remote, states};
use crate::api::auth::AuthUser;
use crate::db::identities::{self, UnlinkResult};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LinkReq {
    pub provider: String,
    /// Куда вернуть игрока после привязки. Пусто — в кабинет.
    pub redirect: Option<String>,
}

/// Начать привязку: отдаём адрес платформы, на который сайт уводит игрока.
///
/// Ссылку выдаёт API, а не собирает сайт: state должен помнить, к какому
/// аккаунту привязка, а токен игрока в адресную строку не положишь.
pub async fn start(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<LinkReq>,
) -> AppResult<Json<Value>> {
    let p = parse_provider(&req.provider)?;
    let creds = config::creds(&state.db, p).await?;
    let csrf = states::create(&state, p, req.redirect.as_deref(), Some(user.user_id)).await?;
    Ok(Json(json!({
        "url": remote::authorize_url(p, &creds, &csrf, &remote::redirect_uri(&state, p))
    })))
}

/// Завершение привязки — сюда приходит общий callback.
pub async fn finish(
    state: &AppState,
    user_id: Uuid,
    p: Provider,
    identity: &RemoteIdentity,
    redirect: Option<String>,
) -> AppResult<Redirect> {
    let linked = identities::link(
        &state.db,
        user_id,
        p.slug(),
        &identity.id,
        &identity.username,
        identity.avatar.as_deref(),
        false,
    )
    .await?;

    let base = redirect.unwrap_or_else(|| format!("{}/cabinet/settings", state.config.web_url));
    let sep = if base.contains('?') { '&' } else { '?' };
    // Занятая привязка — не ошибка сервера: этим аккаунтом уже кто-то входил,
    // и сказать об этом нужно на той странице, откуда игрок начал.
    let tail = if linked {
        format!("linked={}", p.slug())
    } else {
        format!("link_error=taken&provider={}", p.slug())
    };
    if linked {
        crate::audit::record_by_user(
            state,
            user_id,
            crate::audit::actions::AUTH_IDENTITY_LINK,
            json!({ "provider": p.slug(), "provider_user_id": identity.id }),
        )
        .await;
    }
    Ok(Redirect::to(&format!("{base}{sep}{tail}")))
}

/// Список привязок игрока.
pub async fn list(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let items = identities::list(&state.db, user.user_id).await?;
    Ok(Json(json!(items)))
}

/// Отвязать платформу. Ту, через которую регистрировались, — нельзя.
pub async fn unlink(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> AppResult<Json<Value>> {
    let p = parse_provider(&slug)?;
    match identities::unlink(&state.db, user.user_id, p.slug()).await? {
        UnlinkResult::Removed => {
            crate::audit::record_by_user(
                &state,
                user.user_id,
                crate::audit::actions::AUTH_IDENTITY_UNLINK,
                json!({ "provider": p.slug() }),
            )
            .await;
            Ok(Json(json!({ "ok": true })))
        }
        UnlinkResult::NotLinked => Err(AppError::NotFound(format!(
            "{} is not linked to this account",
            p.display_name()
        ))),
        UnlinkResult::Primary => Err(AppError::BadRequest(format!(
            "{} is the account you signed up with — it cannot be unlinked",
            p.display_name()
        ))),
    }
}
