//! A tiny HTTP server for tests: serves fixed responses by path and counts the
//! requests it saw. Enough to exercise download and merge code against a real
//! socket without a mock client.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct Server {
    pub base: String,
    hits: Arc<AtomicUsize>,
    requests: Arc<std::sync::Mutex<Vec<String>>>,
}

impl Server {
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }

    /// The request line and headers of every request so far, oldest first.
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

/// `routes`: path → (status, body). Unknown paths answer 404.
pub async fn serve(routes: Vec<(&str, u16, Vec<u8>)>) -> Server {
    let routes: Arc<HashMap<String, (u16, Vec<u8>)>> = Arc::new(
        routes
            .into_iter()
            .map(|(p, s, b)| (p.to_string(), (s, b)))
            .collect(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    let requests = Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen = requests.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            let routes = routes.clone();
            let seen = seen.clone();
            tokio::spawn(async move {
                let mut req = Vec::new();
                let mut tmp = [0u8; 1024];
                loop {
                    let Ok(n) = sock.read(&mut tmp).await else {
                        return;
                    };
                    req.extend_from_slice(&tmp[..n]);
                    if n == 0 || req.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let line = String::from_utf8_lossy(&req);
                let head_end = line.find("\r\n\r\n").unwrap_or(line.len());
                seen.lock().unwrap().push(line[..head_end].to_string());
                let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
                let (status, body) = routes
                    .get(&path)
                    .cloned()
                    .unwrap_or((404, b"not found".to_vec()));
                let head = format!(
                    "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(&body).await;
                let _ = sock.flush().await;
            });
        }
    });
    Server {
        base: format!("http://{addr}"),
        hits,
        requests,
    }
}

pub fn sha1_hex(data: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    hex::encode(Sha1::digest(data))
}

/// A fresh directory under the system temp dir, removed on drop.
pub struct TempDir(pub std::path::PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("noro-{tag}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
