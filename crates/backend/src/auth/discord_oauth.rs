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
    let Some(code) = request_line
        .strip_prefix("GET /callback?")
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|query| query.split('&').find_map(|kv| kv.strip_prefix("code=")))
    else {
        respond(&mut stream, 404, "text/plain", "Not Found").await?;
        return Ok(None);
    };

    let code = code.to_string();
    respond(&mut stream, 200, "text/html; charset=utf-8", SUCCESS_HTML).await?;
    Ok(Some(code))
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

const SUCCESS_HTML: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8">
<title>Sign in complete</title><style>body{font-family:system-ui,sans-serif;background:#0b1626;
color:#dbe6ff;display:flex;align-items:center;justify-content:center;height:100vh;margin:0}
h1{color:#e85aa5}</style></head><body><div style="text-align:center"><h1>Sign in complete</h1>
<p>You can return to the launcher and close this window.</p></div></body></html>"#;
