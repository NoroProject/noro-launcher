use super::*;
use crate::test_http::serve;

fn api(base: &str) -> MasterApi {
    // A trailing slash on the configured address must not double up.
    MasterApi::new(
        reqwest::Client::new(),
        &format!("{base}/"),
        Some("tok".into()),
    )
    .unwrap()
}

#[tokio::test]
async fn sends_the_session_token_and_reads_the_answer() {
    let server = serve(vec![(
        "/api/notifications/unread",
        200,
        br#"{"unread": 3}"#.to_vec(),
    )])
    .await;
    assert_eq!(api(&server.base).unread_count().await.unwrap(), 3);
    let head = server.requests().remove(0).to_ascii_lowercase();
    assert!(head.starts_with("get /api/notifications/unread "), "{head}");
    assert!(head.contains("authorization: bearer tok"), "{head}");
}

#[tokio::test]
async fn a_refusal_carries_the_masters_reason() {
    let server = serve(vec![(
        "/api/me/punishments",
        403,
        br#"{"error": "staff only"}"#.to_vec(),
    )])
    .await;
    let err = api(&server.base).punishments().await.unwrap_err();
    let text = format!("{err:#}");
    assert!(
        text.contains("403") && text.contains("staff only"),
        "{text}"
    );
}

#[tokio::test]
async fn an_empty_success_is_success() {
    let server = serve(vec![("/api/notifications/read-all", 204, Vec::new())]).await;
    api(&server.base).mark_all_read().await.unwrap();
}

#[tokio::test]
async fn catalog_path_segments_are_escaped() {
    let server = serve(vec![
        (
            "/api/catalog/curse%20forge/project/a%2Fb",
            200,
            br#"{"id": "x"}"#.to_vec(),
        ),
        (
            "/api/catalog/curse%20forge/project/a%2Fb/versions?",
            200,
            b"[]".to_vec(),
        ),
    ])
    .await;
    let api = api(&server.base);
    api.catalog_project("curse forge", "a/b").await.unwrap();
    api.catalog_versions("curse forge", "a/b", "")
        .await
        .unwrap();
}

#[test]
fn nobody_signed_in_means_no_client() {
    assert!(MasterApi::new(reqwest::Client::new(), "http://example.com", None).is_none());
}
