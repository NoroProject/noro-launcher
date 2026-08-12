//! Discord OAuth в лаунчере: поднимаем локальный HTTP-сервер на случайном порту,
//! открываем браузер на мастере, ловим редирект с одноразовым кодом и меняем его
//! на токены запросом к мастеру.
//!
//! Раньше токены присылал сам мастер — POST на `127.0.0.1:{port}`, то есть на
//! свой же localhost: с боевого сервера они не доходили никуда.

use super::token_store::StoredAuth;
use anyhow::{anyhow, bail, Context, Result};
use schema::UserProfile;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct LoginResult {
    pub auth: StoredAuth,
    pub user: UserProfile,
}

/// Выполнить полный OAuth-флоу. `cancelled` позволяет прервать ожидание.
pub async fn login(master_url: &str, cancelled: impl Fn() -> bool) -> Result<LoginResult> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("failed to open local OAuth port")?;
    let port = listener.local_addr()?.port();

    let url = format!(
        "{}/auth/discord/launcher?port={}",
        master_url.trim_end_matches('/'),
        port
    );
    // Открыть браузер (не критично, если не получилось — пользователь скопирует URL).
    if let Err(e) = open::that(&url) {
        tracing::warn!("failed to open browser: {e}; URL: {url}");
    }

    // Ждать callback до 5 минут, периодически проверяя отмену.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(300);
    loop {
        if cancelled() {
            bail!("login cancelled");
        }
        let accept = tokio::time::timeout(Duration::from_millis(500), listener.accept()).await;
        match accept {
            Ok(Ok((stream, _addr))) => {
                if let Some(code) = handle_connection(stream).await? {
                    return exchange(master_url, &code).await;
                }
            }
            Ok(Err(e)) => return Err(anyhow!("accept failed: {e}")),
            Err(_) => {
                // таймаут итерации — проверяем дедлайн/отмену снова
                if tokio::time::Instant::now() >= deadline {
                    bail!("login timed out");
                }
            }
        }
    }
}

fn random_state() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub async fn login_passkey(master_url: &str, cancelled: impl Fn() -> bool) -> Result<LoginResult> {
    login_oauth2(master_url, cancelled).await
}

pub async fn login_oauth2(master_url: &str, cancelled: impl Fn() -> bool) -> Result<LoginResult> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("failed to open local OAuth port")?;
    let port = listener.local_addr()?.port();

    let csrf = random_state();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let base_web_url = std::env::var("NORO_WEB_URL")
        .ok()
        .or_else(|| option_env!("NORO_WEB_URL").map(String::from))
        .unwrap_or_else(|| {
            if master_url.contains("127.0.0.1") || master_url.contains("localhost") {
                "http://localhost:3000".to_string()
            } else {
                "https://noro.dalynkaa.dev".to_string()
            }
        });
    let url = format!(
        "{}/oauth2/authorize?client_id=noro_launcher&redirect_uri={}&response_type=code&scope=profile&state={}",
        base_web_url.trim_end_matches('/'),
        urlencoding::encode(&redirect_uri),
        csrf
    );

    if let Err(e) = open::that(&url) {
        tracing::warn!("failed to open browser: {e}; URL: {url}");
    }

    let deadline = tokio::time::Instant::now() + Duration::from_secs(300);
    loop {
        if cancelled() {
            bail!("login cancelled");
        }
        let accept = tokio::time::timeout(Duration::from_millis(500), listener.accept()).await;
        match accept {
            Ok(Ok((stream, _addr))) => {
                if let Some(code) = handle_connection(stream).await? {
                    return exchange_oauth2(master_url, &code).await;
                }
            }
            Ok(Err(e)) => return Err(anyhow!("accept failed: {e}")),
            Err(_) => {
                if tokio::time::Instant::now() >= deadline {
                    bail!("login timed out");
                }
            }
        }
    }
}

async fn exchange_oauth2(master_url: &str, code: &str) -> Result<LoginResult> {
    #[derive(serde::Deserialize)]
    struct TokenResp {
        access_token: String,
        refresh_token: String,
        user: UserProfile,
    }

    let url = format!(
        "{}/oauth2/token",
        master_url.trim_end_matches('/')
    );
    let resp: TokenResp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "code": code,
            "client_id": "noro_launcher"
        }))
        .send()
        .await
        .context("запрос обмена кода OAuth2")?
        .error_for_status()
        .context("мастер отверг код OAuth2")?
        .json()
        .await
        .context("разбор ответа на обмен кода OAuth2")?;

    Ok(LoginResult {
        auth: StoredAuth {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
        },
        user: resp.user,
    })
}

/// Обработать одно входящее соединение. Возвращает Some с одноразовым кодом.
async fn handle_connection(mut stream: tokio::net::TcpStream) -> Result<Option<String>> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];

    // Нужна только строка запроса: код приезжает в query, тела у GET нет.
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        if find_subsequence(&buf, b"\r\n\r\n").is_some() || buf.len() > 8192 {
            break;
        }
    }

    let head = String::from_utf8_lossy(&buf);
    let request_line = head.lines().next().unwrap_or("");
    let query_opt = request_line
        .strip_prefix("GET /callback?")
        .and_then(|rest| rest.split_whitespace().next());

    if let Some(query) = query_opt {
        if query.contains("error=") {
            respond(&mut stream, 200, "text/html; charset=utf-8", CANCELLED_HTML).await?;
            bail!("Авторизация отменена пользователем");
        }

        if let Some(code) = query.split('&').find_map(|kv| kv.strip_prefix("code=")) {
            let code = code.to_string();
            respond(&mut stream, 200, "text/html; charset=utf-8", SUCCESS_HTML).await?;
            return Ok(Some(code));
        }
    }

    respond(&mut stream, 404, "text/plain", "Not Found").await?;
    Ok(None)
}

/// Обменять код на токены. Идёт к мастеру напрямую, поэтому по HTTPS, и токены
/// не оказываются ни в адресной строке, ни в истории браузера.
async fn exchange(master_url: &str, code: &str) -> Result<LoginResult> {
    #[derive(serde::Deserialize)]
    struct ExchangeResp {
        access_token: String,
        refresh_token: String,
        user: UserProfile,
    }

    let url = format!(
        "{}/auth/launcher/exchange",
        master_url.trim_end_matches('/')
    );
    let resp: ExchangeResp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({ "code": code }))
        .send()
        .await
        .context("запрос обмена кода входа")?
        .error_for_status()
        .context("мастер отверг код входа")?
        .json()
        .await
        .context("разбор ответа на обмен кода")?;

    Ok(LoginResult {
        auth: StoredAuth {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
        },
        user: resp.user,
    })
}

async fn respond(
    stream: &mut tokio::net::TcpStream,
    code: u16,
    content_type: &str,
    body: &str,
) -> Result<()> {
    let resp = format!(
        "HTTP/1.1 {code} OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(resp.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

const SUCCESS_HTML: &str = r#"<!doctype html><html lang="ru"><head><meta charset="utf-8">
<title>Вход выполнен</title><style>body{font-family:system-ui,sans-serif;background:#0b1626;
color:#dbe6ff;display:flex;align-items:center;justify-content:center;height:100vh;margin:0}
h1{color:#e85aa5}</style></head><body><div style="text-align:center"><h1>Вход выполнен</h1>
<p>Вы можете закрыть эту вкладку и вернуться в лаунчер.</p></div></body></html>"#;

const CANCELLED_HTML: &str = r#"<!doctype html><html lang="ru"><head><meta charset="utf-8">
<title>Авторизация отменена</title><style>body{font-family:system-ui,sans-serif;background:#0b1626;
color:#dbe6ff;display:flex;align-items:center;justify-content:center;height:100vh;margin:0}
h1{color:#e85aa5}</style></head><body><div style="text-align:center"><h1>Авторизация отменена</h1>
<p>Вы отклонили запрос доступа. Можете закрыть эту вкладку и вернуться в лаунчер.</p></div></body></html>"#;
