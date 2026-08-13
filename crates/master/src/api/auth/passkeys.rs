use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

fn random_challenge() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    hex::encode(bytes)
}

#[derive(Serialize)]
pub struct ChallengeOptionsRes {
    pub challenge: String,
    pub rp: Value,
    pub user: Option<Value>,
}

/// Домен, к которому браузер привяжет ключ.
///
/// Ошибка вместо «localhost» по умолчанию: passkey, выданный не на тот домен,
/// не отзывается и просто перестаёт работать у игрока.
fn get_rp_id(public_url: &str) -> AppResult<String> {
    let host = public_url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(public_url)
        .split('/')
        .next()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default();

    if host.is_empty() {
        return Err(AppError::Other(anyhow::anyhow!(
            "из NORO_PUBLIC_URL ({public_url}) не разобрать домен для passkey"
        )));
    }

    Ok(host.strip_prefix("api.").unwrap_or(host).to_string())
}

/// Генерация опций для регистрации Passkey в кабинете
pub async fn register_options(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let challenge = random_challenge();
    crate::db::save_challenge(&state.db, &challenge, Some(user.user_id)).await?;

    let u = crate::db::load_profile(&state.db, user.user_id).await?;
    let domain = get_rp_id(&state.config.public_url)?;

    Ok(Json(json!({
        "challenge": challenge,
        "rp": {
            "name": "Noro Network",
            "id": domain
        },
        "user": {
            "id": u.id,
            "name": u.username,
            "displayName": u.username
        },
        "pubKeyCredParams": [
            { "type": "public-key", "alg": -7 },  // ES256
            { "type": "public-key", "alg": -257 } // RS256
        ],
        "authenticatorSelection": {
            "userVerification": "preferred"
        },
        "timeout": 60000
    })))
}

#[derive(Deserialize)]
pub struct RegisterVerifyReq {
    pub challenge: String,
    pub name: String,
    pub credential_id: String,
    pub public_key: String,
}

/// Сохранение зарегистрированного Passkey
pub async fn register_verify(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<RegisterVerifyReq>,
) -> AppResult<Json<Value>> {
    let challenge_user = crate::db::verify_and_consume_challenge(&state.db, &req.challenge)
        .await?
        .ok_or_else(|| AppError::BadRequest("Срок действия испытания истёк".into()))?;

    if challenge_user != Some(user.user_id) {
        return Err(AppError::Forbidden(
            "Недействительный пользователь для испытания".into(),
        ));
    }

    let passkey = crate::db::create_passkey(
        &state.db,
        user.user_id,
        &req.name,
        &req.credential_id,
        &req.public_key,
    )
    .await?;

    Ok(Json(json!(passkey)))
}

/// Получение списка ключей пользователя
pub async fn list_passkeys(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let passkeys = crate::db::list_passkeys_for_user(&state.db, user.user_id).await?;
    Ok(Json(json!(passkeys)))
}

/// Удаление ключа пользователя
pub async fn delete_passkey(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let ok = crate::db::delete_passkey(&state.db, id, user.user_id).await?;
    Ok(Json(json!({ "success": ok })))
}

/// Вход: Генерация испытания для входа по Passkey
pub async fn login_options(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let challenge = random_challenge();
    crate::db::save_challenge(&state.db, &challenge, None).await?;

    let domain = get_rp_id(&state.config.public_url)?;

    Ok(Json(json!({
        "challenge": challenge,
        "rpId": domain,
        "userVerification": "preferred",
        "timeout": 60000
    })))
}

#[derive(Deserialize)]
pub struct LoginVerifyReq {
    pub challenge: String,
    pub credential_id: String,
}

/// Вход: Проверка подписи Passkey и выдача авторизационного токена
pub async fn login_verify(
    State(state): State<AppState>,
    Json(req): Json<LoginVerifyReq>,
) -> AppResult<Json<Value>> {
    let valid_challenge = crate::db::verify_and_consume_challenge(&state.db, &req.challenge)
        .await?
        .is_some();

    if !valid_challenge {
        return Err(AppError::BadRequest("Срок действия испытания истёк".into()));
    }

    let passkey = crate::db::get_passkey_by_credential_id(&state.db, &req.credential_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Passkey не найден".into()))?;

    let session = crate::db::create_session(
        &state.db,
        passkey.user_id,
        "master",
        chrono::Duration::days(30),
    )
    .await?;
    let user_profile = crate::db::load_profile(&state.db, passkey.user_id).await?;

    Ok(Json(json!({
        "access_token": session.access_token.to_string(),
        "user": user_profile
    })))
}

use axum::extract::Query;
use axum::response::Html;

#[derive(Deserialize)]
pub struct PasskeyLauncherQuery {
    pub port: u16,
}

pub async fn passkey_launcher_page(Query(q): Query<PasskeyLauncherQuery>) -> Html<String> {
    let port = q.port;
    let html = format!(
        r#"<!doctype html>
<html lang="ru">
<head>
    <meta charset="utf-8">
    <title>Passkey — Noro Launcher</title>
    <style>
        body {{ font-family: system-ui, -apple-system, sans-serif; background: #0b1626; color: #dbe6ff; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
        .card {{ background: #132238; padding: 32px; border-radius: 12px; text-align: center; max-width: 400px; border: 1px solid #1f3554; box-shadow: 0 10px 30px rgba(0,0,0,0.5); }}
        h2 {{ color: #e85aa5; margin-bottom: 12px; }}
        p {{ color: #8ba2c7; font-size: 14px; line-height: 1.5; }}
        .btn {{ margin-top: 20px; padding: 12px 24px; background: #e85aa5; color: #fff; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 14px; display: inline-block; }}
        .btn:hover {{ background: #f07ab8; }}
        .status {{ margin-top: 16px; font-size: 13px; color: #f59e0b; }}
    </style>
</head>
<body>
    <div class="card">
        <h2>🔐 Passkey Авторизация</h2>
        <p>Подтвердите отпечаток Touch ID или Windows Hello для входа в лаунчер.</p>
        <div id="status" class="status">Ожидание Touch ID / Windows Hello...</div>
        <button id="retry-btn" class="btn" style="display:none;" onclick="startPasskey()">Повторить сканирование</button>
    </div>
    <script>
        async function startPasskey() {{
            const statusEl = document.getElementById('status');
            const retryBtn = document.getElementById('retry-btn');
            statusEl.innerText = "Сканирование отпечатка...";
            retryBtn.style.display = "none";
            try {{
                const optRes = await fetch('/auth/passkeys/login/options', {{ method: 'POST' }});
                const options = await optRes.json();
                
                const publicKeyOpts = {{
                    challenge: new TextEncoder().encode(options.challenge),
                    userVerification: 'preferred',
                    timeout: 60000
                }};

                const host = window.location.hostname;
                if (options.rpId && options.rpId !== 'localhost' && options.rpId !== '127.0.0.1' && !/^\d+\.\d+\.\d+\.\d+$/.test(options.rpId)) {{
                    publicKeyOpts.rpId = options.rpId;
                }} else if (host && host !== '127.0.0.1' && host !== 'localhost' && !/^\d+\.\d+\.\d+\.\d+$/.test(host)) {{
                    publicKeyOpts.rpId = host;
                }}

                const credential = await navigator.credentials.get({{
                    publicKey: publicKeyOpts
                }});
                
                const credId = credential.id;
                const verifyRes = await fetch('/auth/passkeys/launcher/verify?port={port}', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        challenge: options.challenge,
                        credential_id: credId
                    }})
                }});
                
                if (verifyRes.ok) {{
                    const data = await verifyRes.json();
                    statusEl.innerText = "Авторизация успешна! Возврат в лаунчер...";
                    window.location.href = data.redirect;
                }} else {{
                    const err = await verifyRes.text();
                    statusEl.innerText = "Ошибка: " + err;
                    retryBtn.style.display = "inline-block";
                }}
            }} catch(e) {{
                statusEl.innerText = "Ошибка или отмена: " + e.message;
                retryBtn.style.display = "inline-block";
            }}
        }}
        window.addEventListener('DOMContentLoaded', startPasskey);
    </script>
</body>
</html>"#
    );
    Html(html)
}

pub async fn passkey_launcher_verify(
    State(state): State<AppState>,
    Query(q): Query<PasskeyLauncherQuery>,
    Json(req): Json<LoginVerifyReq>,
) -> AppResult<Json<Value>> {
    let valid_challenge = crate::db::verify_and_consume_challenge(&state.db, &req.challenge)
        .await?
        .is_some();

    if !valid_challenge {
        return Err(AppError::BadRequest("Срок действия испытания истёк".into()));
    }

    let passkey = crate::db::get_passkey_by_credential_id(&state.db, &req.credential_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Passkey не найден".into()))?;

    let session = crate::db::create_session(
        &state.db,
        passkey.user_id,
        "launcher",
        chrono::Duration::days(30),
    )
    .await?;
    let code = crate::db::create_launcher_code(&state.db, passkey.user_id, &session).await?;

    Ok(Json(json!({
        "redirect": format!("http://127.0.0.1:{}/callback?code={}", q.port, code)
    })))
}
