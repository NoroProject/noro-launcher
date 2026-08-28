//! Resume doesn't fail loudly when it's wrong, it quietly corrupts the file.
//! The hash has to carry across the join, and a response that ignores Range has
//! to overwrite the partial rather than append to it.

use super::*;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::net::TcpListener;

const PAYLOAD_LEN: usize = 40_000;

fn payload() -> Vec<u8> {
    (0..PAYLOAD_LEN).map(|i| (i % 251) as u8).collect()
}

fn sha1_of(data: &[u8]) -> String {
    hex::encode(Sha1::digest(data))
}

/// Single-shot HTTP server. `honor_range = false` plays an origin that ignores
/// the Range header.
async fn serve_once(body: Vec<u8>, honor_range: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut req = Vec::new();
        let mut tmp = [0u8; 512];
        loop {
            let n = sock.read(&mut tmp).await.unwrap();
            req.extend_from_slice(&tmp[..n]);
            if n == 0 || req.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
        let req = String::from_utf8_lossy(&req).to_ascii_lowercase();
        let start = if honor_range { range_start(&req) } else { None };
        let total = body.len();

        let (head, chunk) = match start {
            Some(s) if s < total => (
                format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {s}-{}/{total}\r\n",
                    total - 1
                ),
                &body[s..],
            ),
            _ => ("HTTP/1.1 200 OK\r\n".to_string(), &body[..]),
        };
        let head = format!(
            "{head}Content-Length: {}\r\nConnection: close\r\n\r\n",
            chunk.len()
        );
        sock.write_all(head.as_bytes()).await.unwrap();
        sock.write_all(chunk).await.unwrap();
        sock.flush().await.unwrap();
    });
    format!("http://{addr}/file")
}

fn range_start(req: &str) -> Option<usize> {
    let line = req.lines().find(|l| l.starts_with("range:"))?;
    line.split("bytes=")
        .nth(1)?
        .split('-')
        .next()?
        .trim()
        .parse()
        .ok()
}

#[tokio::test]
async fn resume_continues_the_hash_across_the_join() {
    let data = payload();
    let dir = tempdir();
    let dest = dir.join("artifact.jar");
    let part = part_path(&dest);
    // Half of it is already on disk from an earlier, interrupted attempt.
    tokio::fs::write(&part, &data[..15_000]).await.unwrap();

    let url = serve_once(data.clone(), true).await;
    let seen = AtomicI64::new(0);
    let on_bytes = |d: i64| {
        seen.fetch_add(d, Ordering::Relaxed);
    };
    fetch_to_file(
        &reqwest::Client::new(),
        &url,
        &dest,
        &sha1_of(&data),
        &on_bytes,
    )
    .await
    .expect("resume should succeed");

    assert_eq!(tokio::fs::read(&dest).await.unwrap(), data);
    assert!(!part.exists(), ".part should have been renamed");
    // Only the tail was counted, not the whole file over again.
    assert_eq!(seen.load(Ordering::Relaxed), (PAYLOAD_LEN - 15_000) as i64);
}

#[tokio::test]
async fn a_200_response_replaces_the_partial_instead_of_appending() {
    let data = payload();
    let dir = tempdir();
    let dest = dir.join("artifact.jar");
    let part = part_path(&dest);
    tokio::fs::write(&part, &data[..15_000]).await.unwrap();

    // Origin with no Range support: sends the whole file.
    let url = serve_once(data.clone(), false).await;
    let seen = AtomicI64::new(0);
    let on_bytes = |d: i64| {
        seen.fetch_add(d, Ordering::Relaxed);
    };
    fetch_to_file(
        &reqwest::Client::new(),
        &url,
        &dest,
        &sha1_of(&data),
        &on_bytes,
    )
    .await
    .expect("should overwrite the partial and match the hash");

    // Appending would give 55000 bytes and a broken sha1.
    assert_eq!(tokio::fs::read(&dest).await.unwrap(), data);
    // The discarded partial rolled back, plus the full download.
    assert_eq!(seen.load(Ordering::Relaxed), PAYLOAD_LEN as i64 - 15_000);
}

#[tokio::test]
async fn a_wrong_hash_removes_the_partial_so_a_retry_starts_clean() {
    let data = payload();
    let dir = tempdir();
    let dest = dir.join("artifact.jar");

    let url = serve_once(data.clone(), true).await;
    let on_bytes = |_: i64| {};
    let err = fetch_to_file(
        &reqwest::Client::new(),
        &url,
        &dest,
        &sha1_of(b"other"),
        &on_bytes,
    )
    .await
    .expect_err("a hash mismatch should be an error");

    assert!(err.to_string().contains("SHA1 mismatch"), "{err}");
    assert!(!dest.exists(), "a corrupt file must not reach dest");
    assert!(
        !part_path(&dest).exists(),
        "the partial has to go, or the retry loops on the same garbage"
    );
}

/// Neighbours download in parallel; a shared `.part` would have them writing
/// over each other, and dest would get the mixture.
#[tokio::test]
async fn neighbours_with_different_extensions_do_not_share_a_partial() {
    let dir = tempdir();
    assert_ne!(
        part_path(&dir.join("SPE_Idol.json")),
        part_path(&dir.join("SPE_Idol.ogg"))
    );
}

fn tempdir() -> std::path::PathBuf {
    let base = std::env::temp_dir().join(format!(
        "noro-fetch-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&base).unwrap();
    base
}
