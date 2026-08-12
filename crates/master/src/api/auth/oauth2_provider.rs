//! Full OAuth 2.0 Provider endpoints for Master.

use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: Option<String>,
    pub state: Option<String>,
}

/// GET /oauth2/authorize — редирект на веб-страницу согласия Nuxt 3
pub async fn authorize_page(
    State(state): State<AppState>,
    axum::extract::RawQuery(query): axum::extract::RawQuery,
) -> AppResult<impl IntoResponse> {
    let web_url = std::env::var("NORO_WEB_URL").unwrap_or_else(|_| {
        if state.config.public_url.contains("127.0.0.1") || state.config.public_url.contains("localhost") {
            "http://localhost:3000".to_string()
        } else {
            "https://noro.dalynkaa.dev".to_string()
        }
    });

    let query_str = query.unwrap_or_default();
    let target = if query_str.is_empty() {
        format!("{}/oauth2/authorize", web_url.trim_end_matches('/'))
    } else {
        format!("{}/oauth2/authorize?{}", web_url.trim_end_matches('/'), query_str)
    };

    Ok(Redirect::temporary(&target))
}

#[derive(Deserialize)]
pub struct AcceptForm {
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: String,
    pub state: String,
}

/// POST /oauth2/authorize/accept — подтверждение согласия
pub async fn accept_authorize(
    State(state): State<AppState>,
    user: AuthUser,
    Json(form): Json<AcceptForm>,
) -> AppResult<Json<Value>> {
    let app = crate::db::get_oauth_app_by_client_id(&state.db, &form.client_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("Приложение не найдено".into()))?;

    crate::db::authorize_app_for_user(&state.db, user.user_id, app.id, &form.scopes).await?;
    let code = crate::db::create_oauth_code(&state.db, user.user_id, app.id, &form.redirect_uri, &form.scopes).await?;

    let sep = if form.redirect_uri.contains('?') { '&' } else { '?' };
    let redirect_target = format!("{}{}code={}&state={}", form.redirect_uri, sep, code, form.state);

    Ok(Json(json!({
        "redirect": redirect_target,
        "code": code.to_string()
    })))
}

#[derive(Deserialize)]
pub struct TokenReq {
    pub grant_type: String,
    pub code: Option<Uuid>,
    pub client_id: String,
    pub client_secret: Option<String>,
}

/// POST /oauth2/token — обмен кода на access_token и refresh_token
pub async fn token_endpoint(
    State(state): State<AppState>,
    Json(req): Json<TokenReq>,
) -> AppResult<Json<Value>> {
    if req.grant_type != "authorization_code" {
        return Err(AppError::BadRequest("Поддерживается только grant_type=authorization_code".into()));
    }

    let Some(code) = req.code else {
        return Err(AppError::BadRequest("Не указан code".into()));
    };

    let (user_id, _app_id, _redirect_uri, scopes) = crate::db::take_oauth_code(&state.db, code)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Код авторизации недействителен или истёк".into()))?;

    let session = crate::db::create_session(&state.db, user_id, &scopes, chrono::Duration::days(30)).await?;
    let profile = crate::db::load_profile(&state.db, user_id).await?;

    Ok(Json(json!({
        "access_token": session.access_token.to_string(),
        "refresh_token": session.refresh_token.to_string(),
        "token_type": "Bearer",
        "expires_in": 2592000,
        "user": profile
    })))
}

/// GET /api/me/authorized-apps — список авторизованных приложений пользователя
pub async fn list_authorized_apps(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let apps = crate::db::list_user_authorized_apps(&state.db, user.user_id).await?;
    Ok(Json(json!(apps)))
}

/// DELETE /api/me/authorized-apps/{app_id} — отозвать доступ приложению
pub async fn revoke_authorized_app(
    State(state): State<AppState>,
    user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let ok = crate::db::revoke_user_authorized_app(&state.db, user.user_id, app_id).await?;
    Ok(Json(json!({ "success": ok })))
}
