//! OAuth2-провайдер: чужие приложения входят игроками нашего инстанса.
//!
//! Проверок здесь три, и все три обязательны — без любой из них поток
//! authorization code перестаёт что-либо гарантировать:
//!
//! * адрес возврата сверяется со списком приложения, иначе код уводят на чужой
//!   домен подменой `redirect_uri`;
//! * `client_secret` проверяется при обмене кода, иначе код меняет на токен
//!   кто угодно, кому он попался в логах или в истории браузера;
//! * запрошенные scope'ы режутся по потолку приложения, иначе разрешение
//!   игрока ничего не значит.

use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::db::oauth_apps::OAuthApp;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// GET /oauth2/authorize — редирект на экран согласия сайта.
pub async fn authorize_page(
    State(state): State<AppState>,
    axum::extract::RawQuery(query): axum::extract::RawQuery,
) -> AppResult<impl IntoResponse> {
    // Адрес сайта берётся из конфига. Угадывать его по подстроке "localhost" в
    // адресе мастера было нельзя: любой прод, кроме одного, уезжал на чужой домен.
    let web_url = &state.config.web_url;

    let query_str = query.unwrap_or_default();
    let target = if query_str.is_empty() {
        format!("{web_url}/oauth2/authorize")
    } else {
        format!("{web_url}/oauth2/authorize?{query_str}")
    };

    Ok(Redirect::temporary(&target))
}

#[derive(Deserialize)]
pub struct ConsentQuery {
    pub client_id: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub scope: Option<String>,
}

/// GET /oauth2/consent — что показать игроку на экране согласия.
///
/// Отдельная ручка, потому что решение принимает игрок: он должен увидеть, кто
/// спрашивает и о чём, до того как что-то подтвердит. Сюда же уезжают отказы —
/// сломанный `redirect_uri` виден на нашей странице, а не после редиректа.
pub async fn consent_info(
    State(state): State<AppState>,
    user: OptionalAuthUser,
    Query(q): Query<ConsentQuery>,
) -> AppResult<Json<Value>> {
    let app = load_app(&state, &q.client_id).await?;
    if !app.allows_redirect(&q.redirect_uri) {
        return Err(AppError::bad(
            crate::error_codes::OAUTH_BAD_REDIRECT,
            "redirect_uri is not registered for this application",
        ));
    }
    // Ровно то, что приложение получит после согласия. Считаем той же функцией,
    // что и выдача: у официального приложения запрошенные scope'ы игнорируются,
    // и экран должен говорить об этом честно — «полный доступ», а не «покажу ник».
    let requested = schema::parse_scopes(&granted_scopes(&app, q.scope.as_deref())?);

    // Владелец видит своё приложение до модерации — иначе интеграцию не собрать.
    let viewer = user.as_ref().map(|u| u.user_id);
    let usable = viewer
        .map(|id| app.usable_by(id))
        .unwrap_or(app.status == crate::db::oauth_apps::STATUS_APPROVED);

    let owner = match app.owner_id {
        Some(id) => crate::db::get_user(&state.db, id)
            .await?
            .map(|u| u.mc_username),
        None => None,
    };

    Ok(Json(json!({
        "name": app.name,
        "icon_url": app.icon_url,
        "description": app.description,
        "official": app.is_official,
        "owner": owner,
        "status": app.status,
        "usable": usable,
        "scopes": describe(&requested),
    })))
}

#[derive(Deserialize)]
pub struct AcceptForm {
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: String,
    #[serde(default)]
    pub state: String,
    /// PKCE: приложение прислало его в самом начале, сайт лишь передаёт дальше.
    #[serde(default)]
    pub code_challenge: Option<String>,
}

/// POST /api/oauth2/authorize/accept — игрок разрешил доступ.
pub async fn accept_authorize(
    State(state): State<AppState>,
    user: AuthUser,
    Json(form): Json<AcceptForm>,
) -> AppResult<Json<Value>> {
    let app = load_app(&state, &form.client_id).await?;
    if !app.usable_by(user.user_id) {
        return Err(AppError::Forbidden(match app.status.as_str() {
            crate::db::oauth_apps::STATUS_PENDING => {
                "this application is still waiting for review".into()
            }
            crate::db::oauth_apps::STATUS_REJECTED => "this application was rejected".to_string(),
            _ => "this application is blocked".to_string(),
        }));
    }
    if !state.oauth_apps_enabled().await && !app.is_official {
        return Err(AppError::Forbidden(
            "third-party applications are turned off on this instance".into(),
        ));
    }
    if !app.allows_redirect(&form.redirect_uri) {
        return Err(AppError::bad(
            crate::error_codes::OAUTH_BAD_REDIRECT,
            "redirect_uri is not registered for this application",
        ));
    }

    // Что игрок подтвердил, то приложение и получит — но не больше потолка.
    let granted = granted_scopes(&app, Some(&form.scopes))?;

    crate::db::authorize_app_for_user(&state.db, user.user_id, app.id, &granted).await?;
    let code = crate::db::create_oauth_code(
        &state.db,
        user.user_id,
        app.id,
        &form.redirect_uri,
        &granted,
        form.code_challenge.as_deref(),
    )
    .await?;

    let sep = if form.redirect_uri.contains('?') {
        '&'
    } else {
        '?'
    };
    let redirect_target = format!(
        "{}{}code={}&state={}",
        form.redirect_uri,
        sep,
        code,
        urlencoding::encode(&form.state)
    );

    Ok(Json(json!({ "redirect": redirect_target })))
}

#[derive(Deserialize)]
pub struct TokenReq {
    pub grant_type: String,
    pub code: Option<Uuid>,
    pub client_id: String,
    pub client_secret: Option<String>,
    /// Тот же адрес, что и при выдаче кода, — как требует RFC 6749 §4.1.3.
    pub redirect_uri: Option<String>,
    /// PKCE: исходная строка, из которой считан challenge.
    pub code_verifier: Option<String>,
}

/// POST /oauth2/token — обмен кода на токен.
pub async fn token_endpoint(
    State(state): State<AppState>,
    Json(req): Json<TokenReq>,
) -> AppResult<Json<Value>> {
    if req.grant_type != "authorization_code" {
        return Err(AppError::BadRequest(
            "Only grant_type=authorization_code is supported".into(),
        ));
    }
    let Some(code) = req.code else {
        return Err(AppError::BadRequest("No code given".into()));
    };
    let app = load_app(&state, &req.client_id).await?;

    // Секрет приложения. Проверяется до всего остального: без него код,
    // подсмотренный в адресной строке, обменивал на токен кто угодно.
    // У публичного клиента секрета нет по определению — его место занимает PKCE.
    if !app.is_public {
        let secret = req
            .client_secret
            .as_deref()
            .ok_or_else(|| AppError::Unauthorized("client_secret is required".into()))?;
        if !crate::api::auth::admin_token::verify(secret, &app.client_secret_hash) {
            return Err(AppError::Unauthorized(
                "client_secret does not match".into(),
            ));
        }
    }

    let taken = crate::db::take_oauth_code(&state.db, code)
        .await?
        .ok_or_else(|| {
            AppError::Unauthorized("The authorization code is invalid or expired".into())
        })?;
    let (user_id, app_id, redirect_uri, scopes) = (
        taken.user_id,
        taken.app_id,
        taken.redirect_uri,
        taken.scopes,
    );

    // PKCE. Требуется, только если авторизация с него началась: сборки лаунчера,
    // выпущенные до его появления, обмениваются кодом по-старому, и запрет
    // сломал бы вход всем, кто ещё не обновился.
    if let Some(challenge) = taken.code_challenge {
        let verifier = req.code_verifier.as_deref().ok_or_else(|| {
            AppError::Unauthorized("code_verifier is required for this code".into())
        })?;
        if !pkce_matches(&challenge, taken.code_challenge_method.as_deref(), verifier) {
            return Err(AppError::Unauthorized(
                "code_verifier does not match the challenge".into(),
            ));
        }
    }

    // Код принадлежит другому приложению — обмен чужого кода на свой токен.
    if app_id != app.id {
        return Err(AppError::Unauthorized(
            "this code was issued to another application".into(),
        ));
    }
    if let Some(sent) = req.redirect_uri.as_deref() {
        if sent != redirect_uri {
            return Err(AppError::Unauthorized(
                "redirect_uri does not match the one used for the code".into(),
            ));
        }
    }

    let session = crate::db::create_app_session(
        &state.db,
        user_id,
        &scopes,
        chrono::Duration::days(30),
        Some(app.id),
    )
    .await?;

    let mut out = json!({
        "access_token": session.access_token.to_string(),
        "refresh_token": session.refresh_token.to_string(),
        "token_type": "Bearer",
        "expires_in": 2592000,
        "scope": scopes,
    });
    // Профиль прямо в ответе — только нашим: лаунчер показывает игрока сразу
    // после входа, и лишний запрос ради этого он делать не должен. Стороннее
    // приложение берёт то же самое на `/api/oauth/me`, ровно по своим scope'ам.
    if schema::is_internal(&scopes) {
        out["user"] = json!(crate::db::load_profile(&state.db, user_id).await?);
    }
    Ok(Json(out))
}

/// Совпадает ли `code_verifier` с сохранённым challenge (RFC 7636).
///
/// `plain` принимаем, но только если приложение само его выбрало: для S256
/// сравниваем base64url(sha256(verifier)) без выравнивания.
fn pkce_matches(challenge: &str, method: Option<&str>, verifier: &str) -> bool {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    match method.unwrap_or("S256") {
        "plain" => challenge == verifier,
        "S256" => {
            let digest = Sha256::digest(verifier.as_bytes());
            let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
            encoded == challenge
        }
        _ => false,
    }
}

/// GET /api/me/authorized-apps — кому игрок открыл доступ.
pub async fn list_authorized_apps(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let apps = crate::db::list_user_authorized_apps(&state.db, user.user_id).await?;
    let items: Vec<Value> = apps
        .into_iter()
        .map(|a| {
            json!({
                "id": a.id,
                "app_id": a.app_id,
                "client_id": a.client_id,
                "name": a.name,
                "icon_url": a.icon_url,
                "description": a.description,
                "authorized_at": a.authorized_at,
                "scopes": describe(&schema::parse_scopes(&a.scopes)),
            })
        })
        .collect();
    Ok(Json(json!(items)))
}

/// DELETE /api/me/authorized-apps/{app_id} — отозвать доступ приложению.
pub async fn revoke_authorized_app(
    State(state): State<AppState>,
    user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let ok = crate::db::revoke_user_authorized_app(&state.db, user.user_id, app_id).await?;
    // Выданные токены умирают вместе с разрешением: иначе «отозвать доступ»
    // означало бы только «не пускать больше», а старый токен продолжал бы жить.
    crate::db::revoke_sessions_for_app(&state.db, user.user_id, app_id).await?;
    Ok(Json(json!({ "success": ok })))
}

/// GET /oauth2/scopes — реестр scope'ов для сайта.
pub async fn list_scopes() -> Json<Value> {
    let items: Vec<Value> = schema::ALL_SCOPES
        .iter()
        .filter(|s| s.tier != schema::ScopeTier::Internal)
        .map(|s| json!({ "name": s.name, "title": s.title, "tier": s.tier }))
        .collect();
    Json(json!(items))
}

async fn load_app(state: &AppState, client_id: &str) -> AppResult<OAuthApp> {
    crate::db::app_by_client_id(&state.db, client_id)
        .await?
        .ok_or_else(|| {
            AppError::bad(
                crate::error_codes::OAUTH_UNKNOWN_CLIENT,
                "unknown client_id",
            )
        })
}

/// Что в итоге получит приложение.
///
/// Официальному выдаётся внутренний scope: наш лаунчер — это и есть аккаунт
/// целиком, дробить его на «покажи ник» бессмысленно. Заодно старые сборки
/// лаунчера продолжают работать: они просят `profile`, а получают то, что им
/// и было нужно всегда.
fn granted_scopes(app: &OAuthApp, raw: Option<&str>) -> AppResult<String> {
    if app.is_official {
        return Ok(schema::SCOPE_LAUNCHER.to_string());
    }
    Ok(requested_scopes(app, raw)?.join(" "))
}

/// Запрошенные scope'ы, обрезанные потолком приложения.
///
/// Пустой запрос — это `identity`: приложению всё равно нужно знать, кто вошёл,
/// а молчаливая выдача всего подряд как раз и была старым поведением.
fn requested_scopes(app: &OAuthApp, raw: Option<&str>) -> AppResult<Vec<String>> {
    let allowed = app.allowed_scope_list();
    let asked = schema::parse_scopes(raw.unwrap_or(""));
    if asked.is_empty() {
        return Ok(vec![schema::SCOPE_IDENTITY.to_string()]);
    }
    for scope in &asked {
        if schema::grantable(scope).is_none() {
            return Err(AppError::bad(
                crate::error_codes::OAUTH_BAD_SCOPE,
                format!("unknown scope: {scope}"),
            ));
        }
        if !allowed.iter().any(|a| a == scope) {
            return Err(AppError::bad(
                crate::error_codes::OAUTH_BAD_SCOPE,
                format!("this application may not request the {scope} scope"),
            ));
        }
    }
    Ok(asked)
}

/// Scope'ы вместе с формулировками — экран согласия показывает их игроку.
fn describe(scopes: &[String]) -> Vec<Value> {
    scopes
        .iter()
        .map(|name| {
            // Ищем по всему реестру, включая внутренние: у нашего лаунчера scope
            // именно внутренний, и игрок должен прочитать про полный доступ.
            match schema::ALL_SCOPES.iter().find(|s| s.name == name) {
                Some(s) => json!({ "name": s.name, "title": s.title, "tier": s.tier }),
                None => json!({ "name": name, "title": name, "tier": "basic" }),
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "oauth2_provider_tests.rs"]
mod tests;
