//! Discord OAuth в лаунчере: поднимаем локальный HTTP-сервер на случайном порту,
//! открываем браузер на мастере, ждём POST с токенами на localhost.

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
                if let Some(result) = handle_connection(stream).await? {
                    return Ok(result);
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

#[derive(serde::Deserialize)]
struct CallbackPayload {
    access_token: String,
    refresh_token: String,
    user: UserProfile,
}

/// Обработать одно входящее соединение. Возвращает Some при успешном /callback.
async fn handle_connection(mut stream: tokio::net::TcpStream) -> Result<Option<LoginResult>> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];

    // Читаем до конца заголовков, затем тело по Content-Length.
    let mut header_end = None;
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_subsequence(&buf, b"\r\n\r\n") {
            header_end = Some(pos + 4);
            break;
        }
        if buf.len() > 1024 * 1024 {
            break;
        }
    }

    let Some(header_end) = header_end else {
        respond(&mut stream, 400, "Bad Request").await?;
        return Ok(None);
    };

    let headers = String::from_utf8_lossy(&buf[..header_end]);
    let first_line = headers.lines().next().unwrap_or("");
    // Только POST /callback нас интересует.
    if !first_line.starts_with("POST /callback") {
        respond(&mut stream, 404, "Not Found").await?;
        return Ok(None);
    }

    let content_length: usize = headers
        .lines()
        .find_map(|l| {
            let l = l.to_ascii_lowercase();
            l.strip_prefix("content-length:")
                .map(|v| v.trim().parse().unwrap_or(0))
        })
        .unwrap_or(0);

    // Дочитать тело.
    while buf.len() < header_end + content_length {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
    }
    let body = &buf[header_end..(header_end + content_length).min(buf.len())];

    match serde_json::from_slice::<CallbackPayload>(body) {
        Ok(payload) => {
            respond(&mut stream, 200, "OK").await?;
            Ok(Some(LoginResult {
                auth: StoredAuth {
                    access_token: payload.access_token,
                    refresh_token: payload.refresh_token,
                },
                user: payload.user,
            }))
        }
        Err(e) => {
            respond(&mut stream, 400, "Bad JSON").await?;
            Err(anyhow!("failed to parse callback: {e}"))
        }
    }
}

async fn respond(stream: &mut tokio::net::TcpStream, code: u16, msg: &str) -> Result<()> {
    let body = format!("{code} {msg}");
    let resp = format!(
        "HTTP/1.1 {code} {msg}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
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
