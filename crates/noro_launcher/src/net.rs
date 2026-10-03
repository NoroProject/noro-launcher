//! HTTP for the bootstrapper: bounded waits and a few retries.
//!
//! Without timeouts a half-open connection — a sleeping laptop, a captive
//! portal, a proxy that swallows the request — kept the splash on one number
//! forever, and the only way out was the task manager.

use std::future::Future;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Between two reads of the body, not for the whole download: a slow link
/// still finishes, a dead one gives up.
const READ_TIMEOUT: Duration = Duration::from_secs(30);
/// The update check runs before any window is up, so the player is looking
/// at nothing while it waits. Better to start what is installed.
const QUICK_TIMEOUT: Duration = Duration::from_secs(5);
const ATTEMPTS: u32 = 3;

pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .build()
        .unwrap_or_default()
}

/// For the version check before the window: one short try.
pub fn quick_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(QUICK_TIMEOUT)
        .timeout(QUICK_TIMEOUT)
        .build()
        .unwrap_or_default()
}

/// Worth another try: the link or the master hiccuped. A 404 or a body that
/// isn't JSON will be the same a second later.
pub fn transient(e: &reqwest::Error) -> bool {
    match e.status() {
        Some(status) => status.is_server_error() || status.as_u16() == 429,
        None => !e.is_decode() && !e.is_builder(),
    }
}

/// Runs `f` up to three times, waiting 1 s and then 2 s between attempts.
pub async fn retry<T, F, Fut>(mut f: F) -> reqwest::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = reqwest::Result<T>>,
{
    let mut delay = Duration::from_secs(1);
    let mut attempt = 1;
    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) if attempt < ATTEMPTS && transient(&e) => {
                crate::log::line(&format!("attempt {attempt} failed, retrying: {e}"));
                tokio::time::sleep(delay).await;
                delay *= 2;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Answers each connection with the next status in `statuses`, then 200.
    async fn server(statuses: Vec<u16>) -> (String, std::sync::Arc<AtomicU32>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let hits = std::sync::Arc::new(AtomicU32::new(0));
        let counter = hits.clone();
        tokio::spawn(async move {
            while let Ok((mut sock, _)) = listener.accept().await {
                let n = counter.fetch_add(1, Ordering::SeqCst) as usize;
                let status = statuses.get(n).copied().unwrap_or(200);
                let mut buf = [0u8; 1024];
                let _ = sock.read(&mut buf).await;
                let reply = format!(
                    "HTTP/1.1 {status} X\r\ncontent-length: 2\r\nconnection: close\r\n\r\nok"
                );
                let _ = sock.write_all(reply.as_bytes()).await;
            }
        });
        (url, hits)
    }

    async fn fetch(client: &reqwest::Client, url: &str) -> reqwest::Result<String> {
        client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    #[tokio::test]
    async fn a_server_error_is_retried() {
        let (url, hits) = server(vec![503]).await;
        let client = client();
        let body = retry(|| fetch(&client, &url)).await.unwrap();
        assert_eq!(body, "ok");
        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_missing_file_is_not() {
        let (url, hits) = server(vec![404, 404, 404]).await;
        let client = client();
        assert!(retry(|| fetch(&client, &url)).await.is_err());
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }
}
