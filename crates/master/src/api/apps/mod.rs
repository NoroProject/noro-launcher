//! Свои OAuth2-приложения игрока: завести, поправить, показать секрет.
//!
//! Владелец распоряжается приложением сам, но опубликовать его — решение
//! оператора: до одобрения приложение впускает только автора (см.
//! `OAuthApp::usable_by`).

mod icon;

use crate::api::auth::{admin_token, AuthUser};
use crate::db::oauth_apps::{NewApp, OAuthApp};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::Json;
use rand::Rng;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Сколько приложений может завести один игрок.
///
/// Не про нагрузку, а про очередь модерации: без потолка один человек за вечер
/// накидывает туда сотню заявок, и разбирать их будет уже не оператор, а никто.
const APPS_PER_USER: usize = 10;

/// GET /api/me/apps — мои приложения.
pub async fn list(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let apps = crate::db::apps_by_owner(&state.db, user.user_id).await?;
    let items: Vec<Value> = apps.iter().map(view).collect();
    let enabled = state.oauth_apps_enabled().await;
    Ok(Json(json!({
        "items": items,
        // Выключены сторонние приложения — в кабинете исчезает весь раздел, а
        // не только кнопка: управлять тем, что всё равно никого не впустит,
        // незачем.
        "enabled": enabled,
        // Кнопка «создать» пропадает вместе с разрешением — и сервер откажет
        // тоже, а не только сайт спрячет.
        "can_create": enabled
            && state.oauth_apps_creation_enabled().await
            && apps.len() < APPS_PER_USER,
        "limit": APPS_PER_USER,
    })))
}

#[derive(Deserialize)]
pub struct AppForm {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Адреса возврата, по одному в строке.
    pub redirect_uris: String,
    /// Что приложение будет просить у игроков. Только базовые — привилегированные
    /// выдаёт оператор, и здесь их не назначить.
    #[serde(default)]
    pub scopes: Vec<String>,
}

impl AppForm {
    /// Проверка формы до записи.
    ///
    /// Адрес возврата обязателен и обязан быть http(s): без него приложение
    /// нерабочее, а `javascript:`-адрес превращает экран согласия в чужой
    /// скрипт на нашем домене.
    fn check(&self) -> AppResult<(String, Option<String>, String)> {
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > 100 {
            return Err(AppError::BadRequest(
                "the name must be 1 to 100 characters".into(),
            ));
        }
        let mut uris = Vec::new();
        for line in self.redirect_uris.lines() {
            let uri = line.trim();
            if uri.is_empty() {
                continue;
            }
            let parsed = url::Url::parse(uri)
                .map_err(|e| AppError::BadRequest(format!("{uri} is not a URL: {e}")))?;
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err(AppError::BadRequest(format!(
                    "{uri}: only http and https redirects are allowed"
                )));
            }
            if parsed.fragment().is_some() {
                return Err(AppError::BadRequest(format!(
                    "{uri}: a redirect must not contain a #fragment"
                )));
            }
            uris.push(parsed.to_string());
        }
        if uris.is_empty() {
            return Err(AppError::BadRequest(
                "at least one redirect URI is required".into(),
            ));
        }

        let description = self
            .description
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
            .map(str::to_string);
        if description
            .as_deref()
            .is_some_and(|d| d.chars().count() > 500)
        {
            return Err(AppError::BadRequest(
                "the description must be 500 characters or shorter".into(),
            ));
        }
        Ok((name.to_string(), description, uris.join("\n")))
    }

    /// Что приложение будет просить.
    ///
    /// Пустой выбор — это `identity`: приложение обязано узнать, кто вошёл, а
    /// «ничего» в списке доступов на экране согласия выглядит как поломка.
    /// Привилегированные scope'ы владелец назначить не может: их смысл в том,
    /// что за ними стоит решение оператора.
    fn check_scopes(&self) -> AppResult<Vec<String>> {
        let mut out = Vec::new();
        for scope in &self.scopes {
            let known = schema::grantable(scope).ok_or_else(|| {
                AppError::bad(
                    crate::error_codes::OAUTH_BAD_SCOPE,
                    format!("unknown scope: {scope}"),
                )
            })?;
            if known.tier != schema::ScopeTier::Basic {
                return Err(AppError::bad(
                    crate::error_codes::OAUTH_BAD_SCOPE,
                    format!("the {scope} scope is granted by the staff, not by you"),
                ));
            }
            if !out.iter().any(|s| s == scope) {
                out.push(scope.clone());
            }
        }
        if out.is_empty() {
            out.push(schema::SCOPE_IDENTITY.to_string());
        }
        Ok(out)
    }
}

/// POST /api/me/apps — завести приложение. Секрет показывается один раз.
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(form): Json<AppForm>,
) -> AppResult<Json<Value>> {
    if !state.oauth_apps_enabled().await || !state.oauth_apps_creation_enabled().await {
        return Err(AppError::Forbidden(
            "creating applications is turned off on this instance".into(),
        ));
    }
    let mine = crate::db::apps_by_owner(&state.db, user.user_id).await?;
    if mine.len() >= APPS_PER_USER {
        return Err(AppError::BadRequest(format!(
            "you already have {APPS_PER_USER} applications"
        )));
    }
    let (name, description, redirect_uris) = form.check()?;
    let scopes = form.check_scopes()?;

    let secret = generate_secret();
    let app = crate::db::create_app(
        &state.db,
        NewApp {
            owner_id: user.user_id,
            client_id: &generate_client_id(),
            secret_hash: &admin_token::hash(&secret).map_err(AppError::Other)?,
            name: &name,
            description: description.as_deref(),
            redirect_uris: &redirect_uris,
            scopes: &scopes.join(" "),
        },
    )
    .await?;

    let mut out = view(&app);
    // Единственный раз, когда секрет виден: в базе лежит только его argon2-хеш.
    out["client_secret"] = json!(secret);
    Ok(Json(out))
}

/// PUT /api/me/apps/{id} — поправить своё приложение.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(form): Json<AppForm>,
) -> AppResult<Json<Value>> {
    let app = mine(&state, &user, id).await?;
    let (name, description, redirect_uris) = form.check()?;
    let updated = crate::db::update_app(
        &state.db,
        app.id,
        &name,
        description.as_deref(),
        &redirect_uris,
    )
    .await?
    .ok_or_else(|| AppError::NotFound("application".into()))?;

    // Привилегированные scope'ы остаются: их выдал оператор, и правка названия
    // владельцем не повод молча их снимать.
    let mut scopes = form.check_scopes()?;
    for granted in app.allowed_scope_list() {
        let privileged =
            schema::grantable(&granted).is_some_and(|s| s.tier == schema::ScopeTier::Privileged);
        if privileged && !scopes.contains(&granted) {
            scopes.push(granted);
        }
    }
    let updated = crate::db::set_allowed_scopes(&state.db, app.id, &scopes.join(" "))
        .await?
        .unwrap_or(updated);
    Ok(Json(view(&updated)))
}

/// POST /api/me/apps/{id}/secret — выпустить новый секрет вместо утёкшего.
pub async fn rotate_secret(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let app = mine(&state, &user, id).await?;
    let secret = generate_secret();
    crate::db::set_secret_hash(
        &state.db,
        app.id,
        &admin_token::hash(&secret).map_err(AppError::Other)?,
    )
    .await?;
    Ok(Json(json!({ "client_secret": secret })))
}

/// POST /api/me/apps/{id}/icon — залить иконку файлом.
pub async fn upload_icon(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<Value>> {
    let app = mine(&state, &user, id).await?;
    let url = store_icon(&state, app.id, multipart).await?;
    Ok(Json(json!({ "icon_url": url })))
}

/// Принять иконку, привести к общему виду и записать приложению.
///
/// Общая и для кабинета, и для админки: у официальных приложений владельца нет,
/// но иконка нужна ровно та же — и обрабатываться она должна одинаково, иначе
/// у половины приложений на экране согласия окажется свой формат.
pub async fn store_icon(
    state: &AppState,
    app_id: Uuid,
    mut multipart: Multipart,
) -> AppResult<String> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if !matches!(field.name().unwrap_or_default(), "image" | "file" | "icon") {
            continue;
        }
        let raw = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        let data = tokio::task::spawn_blocking(move || icon::prepare(&raw))
            .await
            .map_err(|e| AppError::Other(e.into()))?
            .map_err(|e| {
                AppError::bad(
                    crate::error_codes::UPLOAD_BAD_FORMAT,
                    format!("invalid icon: {e}"),
                )
            })?;

        let stored = state
            .files
            .put_bytes(&data)
            .await
            .map_err(AppError::Other)?;
        let url = if let Some(s3) = &state.config.s3 {
            crate::files::s3::put(state.http(), s3, &stored.sha1, &data)
                .await
                .map_err(AppError::Other)?
        } else {
            state.config.file_url(&stored.sha1)
        };
        crate::db::set_icon(&state.db, app_id, &url).await?;
        return Ok(url);
    }
    Err(AppError::bad(
        crate::error_codes::UPLOAD_FIELD_MISSING,
        "missing the image field",
    ))
}

/// DELETE /api/me/apps/{id} — удалить своё приложение вместе с доступами.
pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let app = mine(&state, &user, id).await?;
    crate::db::revoke_all_sessions_for_app(&state.db, app.id).await?;
    let removed = crate::db::delete_app(&state.db, app.id).await?;
    Ok(Json(json!({ "ok": removed })))
}

/// Приложение, которым игрок вправе распоряжаться.
///
/// Официальные не отдаём даже владельцу-оператору: ими управляют из админки,
/// где действие попадает в журнал.
async fn mine(state: &AppState, user: &AuthUser, id: Uuid) -> AppResult<OAuthApp> {
    let app = crate::db::app_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("application".into()))?;
    if app.owner_id != Some(user.user_id) || app.is_official {
        return Err(AppError::NotFound("application".into()));
    }
    Ok(app)
}

/// Приложение так, как его видит владелец. Хеш секрета наружу не уходит.
fn view(app: &OAuthApp) -> Value {
    json!({
        "id": app.id,
        "client_id": app.client_id,
        "name": app.name,
        "description": app.description,
        "icon_url": app.icon_url,
        "redirect_uris": app.redirect_uris,
        "status": app.status,
        "review_note": app.review_note,
        "allowed_scopes": app.allowed_scope_list(),
        "is_official": app.is_official,
        "created_at": app.created_at,
        "updated_at": app.updated_at,
    })
}

fn generate_client_id() -> String {
    let bytes: [u8; 12] = rand::thread_rng().gen();
    format!("app_{}", hex::encode(bytes))
}

/// `cs` — client secret. Префикс отличает его от admin-токенов и кодов, которые
/// тоже начинаются с `noro_`.
fn generate_secret() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    format!("noro_cs_{}", hex::encode(bytes))
}

#[cfg(test)]
#[path = "form_tests.rs"]
mod form_tests;
