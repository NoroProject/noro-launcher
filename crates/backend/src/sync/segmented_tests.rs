//! A file glued together from ranges has to be byte-for-byte the file, and a
//! server that ignores ranges must leave nothing behind for the fallback.

use super::*;
use sha1::{Digest, Sha1};
use std::sync::atomic::AtomicI64;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const LEN: usize = 100_003;

fn payload() -> Vec<u8> {
    (0..LEN).map(|i| (i * 7 % 251) as u8).collect()
}

/// Answers any number of requests; `ranges = false` plays an origin that
/// ignores Range and always sends the whole body.
async fn serve(body: Vec<u8>, ranges: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let body = body.clone();
            tokio::spawn(async move {
                let mut req = Vec::new();
                let mut tmp = [0u8; 512];
                while !req.windows(4).any(|w| w == b"\r\n\r\n") {
                    let n = sock.read(&mut tmp).await.unwrap();
                    if n == 0 {
                        return;
                    }
                    req.extend_from_slice(&tmp[..n]);
                }
                let req = String::from_utf8_lossy(&req).to_ascii_lowercase();
                let range = req
                    .lines()
                    .find_map(|l| l.strip_prefix("range: bytes="))
                    .and_then(|r| {
                        let (a, b) = r.trim().split_once('-')?;
                        Some((a.parse::<usize>().ok()?, b.parse::<usize>().ok()?))
                    });
                let total = body.len();
                let (head, chunk) = match range {
                    Some((a, b)) if ranges => (
                        format!("HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {a}-{b}/{total}\r\n"),
                        &body[a..=b],
                    ),
                    _ => ("HTTP/1.1 200 OK\r\n".to_string(), &body[..]),
                };
                let head = format!(
                    "{head}Content-Length: {}\r\nConnection: close\r\n\r\n",
                    chunk.len()
                );
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(chunk).await;
            });
        }
    });
    format!("http://{addr}/file")
}

fn pool() -> HttpClientPool {
    HttpClientPool::new(3).unwrap()
}

#[tokio::test]
async fn ranges_over_several_connections_make_the_whole_file() {
    let data = payload();
    let url = serve(data.clone(), true).await;
    let dir = crate::test_http::TempDir::new("segmented");
    let dest = dir.path().join("big.bin");
    let seen = AtomicI64::new(0);
    let on_bytes = |d: i64| {
        seen.fetch_add(d, Ordering::Relaxed);
    };
    let sha = hex::encode(Sha1::digest(&data));
    let stats = fetch_split(
        &pool(),
        &url,
        &dest,
        &sha,
        LEN as u64,
        3,
        &on_bytes,
        &|| false,
    )
    .await
    .unwrap()
    .expect("the server honours ranges");
    assert_eq!(stats.segments, 3);
    assert_eq!(std::fs::read(&dest).unwrap(), data);
    assert!(!part_path(&dest).exists());
    assert_eq!(seen.load(Ordering::Relaxed), LEN as i64);
}

#[tokio::test]
async fn a_server_without_ranges_hands_back_to_the_plain_download() {
    let data = payload();
    let url = serve(data.clone(), false).await;
    let dir = crate::test_http::TempDir::new("segmented");
    let dest = dir.path().join("big.bin");
    let seen = AtomicI64::new(0);
    let on_bytes = |d: i64| {
        seen.fetch_add(d, Ordering::Relaxed);
    };
    let sha = hex::encode(Sha1::digest(&data));
    let out = fetch_split(
        &pool(),
        &url,
        &dest,
        &sha,
        LEN as u64,
        3,
        &on_bytes,
        &|| false,
    )
    .await
    .unwrap();
    assert!(out.is_none());
    assert!(!dest.exists() && !part_path(&dest).exists());
    assert_eq!(
        seen.load(Ordering::Relaxed),
        0,
        "progress must be rolled back"
    );
}

#[tokio::test]
async fn a_wrong_hash_leaves_nothing_and_rolls_the_bar_back() {
    let data = payload();
    let url = serve(data, true).await;
    let dir = crate::test_http::TempDir::new("segmented");
    let dest = dir.path().join("big.bin");
    let seen = AtomicI64::new(0);
    let on_bytes = |d: i64| {
        seen.fetch_add(d, Ordering::Relaxed);
    };
    let err = fetch_split(
        &pool(),
        &url,
        &dest,
        &"0".repeat(40),
        LEN as u64,
        3,
        &on_bytes,
        &|| false,
    )
    .await
    .unwrap_err();
    assert!(format!("{err:#}").contains("SHA1 mismatch"), "{err:#}");
    assert!(!dest.exists() && !part_path(&dest).exists());
    assert_eq!(seen.load(Ordering::Relaxed), 0);
}
