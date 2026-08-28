//! Signing in through the website.
//!
//! The launcher opens a local HTTP server on a random port, sends the browser
//! to the master, which hands off to the site's consent page, and a one-time
//! code comes back to that port. The launcher then fetches the tokens itself.
//!
//! There are deliberately no "sign in with X" buttons here. The site already
//! has all of them and can add more without anyone reinstalling the launcher.

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

fn random_state() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// PKCE (RFC 7636). The launcher can't hold a client secret — it runs on the
/// player's machine — so the code arriving on the local port is worthless on its
/// own: only whoever knows the verifier can trade it in. Otherwise any other
/// process listening on loopback could race for it.
fn pkce_pair() -> (String, String) {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

/// The whole flow. `cancelled` is polled while waiting, so the player can back
/// out without waiting for the five-minute timeout.
pub async fn login(master_url: &str, cancelled: impl Fn() -> bool) -> Result<LoginResult> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("failed to open local OAuth port")?;
    let port = listener.local_addr()?.port();

    let csrf = random_state();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    // Point the browser at the master, not the site: `/oauth2/authorize`
    // redirects to whatever `NORO_WEB_URL` the master is configured with, so
    // the launcher never needs to know the site's address and a third-party
    // deployment keeps its own players.
    let (verifier, challenge) = pkce_pair();
    let url = format!(
        "{}/oauth2/authorize?client_id=noro_launcher&redirect_uri={}&response_type=code\
         &scope=launcher&state={}&code_challenge={}&code_challenge_method=S256",
        master_url.trim_end_matches('/'),
        urlencoding::encode(&redirect_uri),
        csrf,
        challenge
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
                    return exchange(master_url, &code, &verifier).await;
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

/// Goes to the master directly, over HTTPS, so the tokens never touch the
/// address bar or the browser's history.
async fn exchange(master_url: &str, code: &str, verifier: &str) -> Result<LoginResult> {
    #[derive(serde::Deserialize)]
    struct TokenResp {
        access_token: String,
        refresh_token: String,
        user: UserProfile,
    }

    let url = format!("{}/oauth2/token", master_url.trim_end_matches('/'));
    let resp: TokenResp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "code": code,
            "client_id": "noro_launcher",
            "code_verifier": verifier
        }))
        .send()
        .await
        .context("OAuth2 code exchange request")?
        .error_for_status()
        .context("master rejected the OAuth2 code")?
        .json()
        .await
        .context("parsing the OAuth2 token response")?;

    Ok(LoginResult {
        auth: StoredAuth {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
        },
        user: resp.user,
    })
}

/// `Some` when this connection carried the one-time code. Anything else on the
/// port gets a 404 and the loop keeps waiting.
async fn handle_connection(mut stream: tokio::net::TcpStream) -> Result<Option<String>> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];

    // Only the request line matters — the code is in the query and a GET has no
    // body — so stop at the end of the headers or at 8 KiB, whichever is first.
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
            bail!("authorization denied by the user");
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
