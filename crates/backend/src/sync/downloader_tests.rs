use super::*;
use crate::test_http::{serve, sha1_hex, TempDir};
use std::sync::atomic::AtomicU64;

fn task(server: &crate::test_http::Server, path: &str, body: &[u8], dest: PathBuf) -> DownloadTask {
    DownloadTask {
        url: server.url(path),
        dest,
        sha1: sha1_hex(body),
        size: body.len() as u64,
        executable: false,
    }
}

#[tokio::test]
async fn downloads_every_file_and_reports_the_exact_total() {
    let a = b"first file".to_vec();
    let b = b"the second one".to_vec();
    let server = serve(vec![("/a", 200, a.clone()), ("/b", 200, b.clone())]).await;
    let tmp = TempDir::new("downloader");
    let tasks = vec![
        task(&server, "/a", &a, tmp.path().join("a.bin")),
        task(&server, "/b", &b, tmp.path().join("nested/b.bin")),
    ];
    let last = Arc::new(AtomicU64::new(0));
    let seen = last.clone();
    download_all(
        &reqwest::Client::new(),
        tasks,
        2,
        move |done| seen.store(done, Ordering::SeqCst),
        || false,
    )
    .await
    .unwrap();
    assert_eq!(std::fs::read(tmp.path().join("a.bin")).unwrap(), a);
    assert_eq!(std::fs::read(tmp.path().join("nested/b.bin")).unwrap(), b);
    assert_eq!(last.load(Ordering::SeqCst), (a.len() + b.len()) as u64);
}

#[tokio::test]
async fn a_file_that_never_arrives_fails_the_stage_with_its_address() {
    let server = serve(vec![]).await;
    let tmp = TempDir::new("downloader");
    let tasks = vec![task(&server, "/missing", b"x", tmp.path().join("x"))];
    let err = download_all(&reqwest::Client::new(), tasks, 1, |_| {}, || false)
        .await
        .unwrap_err();
    assert!(format!("{err:#}").contains("/missing"), "{err:#}");
    assert_eq!(server.hits() as u32, MAX_ATTEMPTS);
}

#[tokio::test]
async fn cancelled_before_starting_fetches_nothing() {
    let server = serve(vec![("/a", 200, b"a".to_vec())]).await;
    let tmp = TempDir::new("downloader");
    let tasks = vec![task(&server, "/a", b"a", tmp.path().join("a"))];
    assert!(
        download_all(&reqwest::Client::new(), tasks, 1, |_| {}, || true)
            .await
            .is_err()
    );
    assert_eq!(server.hits(), 0);
}

#[tokio::test]
async fn needs_download_trusts_size_unless_asked_to_hash() {
    let tmp = TempDir::new("downloader");
    let cache = HashCache::load(tmp.path()).await;
    let path = tmp.path().join("f");
    let body = b"contents";
    assert!(needs_download(&path, 8, &sha1_hex(body), true, &cache).await);

    std::fs::write(&path, body).unwrap();
    assert!(needs_download(&path, 9, &sha1_hex(body), false, &cache).await);
    assert!(!needs_download(&path, 8, "wrong", false, &cache).await);
    assert!(needs_download(&path, 8, "wrong", true, &cache).await);
    assert!(!needs_download(&path, 8, &sha1_hex(body), true, &cache).await);
}
