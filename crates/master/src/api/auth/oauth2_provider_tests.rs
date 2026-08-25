//! Проверки, без которых поток authorization code ничего не гарантирует.
//!
//! Всё, что здесь описано, раньше работало наоборот: адрес возврата не
//! сверялся, scope'ы не ограничивались, PKCE не было.

use super::*;
use crate::db::oauth_apps::{OAuthApp, STATUS_APPROVED, STATUS_PENDING, STATUS_SUSPENDED};
use chrono::Utc;
use uuid::Uuid;

fn app(redirects: &str, allowed: &str) -> OAuthApp {
    OAuthApp {
        id: Uuid::nil(),
        client_id: "app_test".into(),
        client_secret_hash: "hash".into(),
        name: "Test".into(),
        icon_url: None,
        description: None,
        redirect_uris: redirects.into(),
        is_official: false,
        is_public: false,
        owner_id: Some(Uuid::from_u128(7)),
        status: STATUS_APPROVED.into(),
        allowed_scopes: allowed.into(),
        review_note: None,
        reviewed_by: None,
        reviewed_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn a_redirect_must_match_exactly() {
    let a = app("https://app.example/cb", "identity");
    assert!(a.allows_redirect("https://app.example/cb"));
    assert!(!a.allows_redirect("https://app.example/cb/../evil"));
    assert!(!a.allows_redirect("https://app.example.evil.tld/cb"));
    assert!(!a.allows_redirect("https://app.example/cb2"));
}

#[test]
fn only_a_public_client_gets_the_loopback_exception() {
    let mut a = app("http://127.0.0.1", "identity");
    // Обычному приложению порт не прощается: у него нет причин слушать петлю.
    assert!(!a.allows_redirect("http://127.0.0.1:53123/callback"));

    a.is_public = true;
    assert!(a.allows_redirect("http://127.0.0.1:53123/callback"));
    // Чужой хост петлёй не становится, как бы он ни назывался.
    assert!(!a.allows_redirect("http://127.0.0.1.evil.tld/callback"));
    assert!(!a.allows_redirect("https://example.com/callback"));
}

#[test]
fn an_application_cannot_ask_beyond_its_ceiling() {
    let a = app("https://app.example/cb", "identity profile");
    assert_eq!(
        requested_scopes(&a, Some("identity profile")).unwrap(),
        ["identity", "profile"]
    );
    // Привилегированный scope не выдан оператором — запрос отклоняется целиком.
    assert!(requested_scopes(&a, Some("identity skins:write")).is_err());
    // Выдуманный scope тоже: молча выкинуть его значит выдать не то, что видел игрок.
    assert!(requested_scopes(&a, Some("identity everything")).is_err());
}

#[test]
fn an_empty_request_means_identity_only() {
    let a = app("https://app.example/cb", "identity profile skins");
    assert_eq!(requested_scopes(&a, None).unwrap(), ["identity"]);
    assert_eq!(requested_scopes(&a, Some("  ")).unwrap(), ["identity"]);
}

#[test]
fn our_own_application_gets_the_internal_scope() {
    let mut a = app("http://127.0.0.1", "identity");
    a.is_official = true;
    // Что бы ни попросила старая сборка лаунчера, выдаётся внутренний scope.
    assert_eq!(granted_scopes(&a, Some("profile")).unwrap(), "launcher");
}

#[test]
fn a_third_party_application_never_gets_an_internal_scope() {
    let a = app("https://app.example/cb", "identity launcher");
    assert!(requested_scopes(&a, Some("launcher")).is_err());
}

#[test]
fn an_unapproved_application_only_serves_its_author() {
    let mut a = app("https://app.example/cb", "identity");
    a.status = STATUS_PENDING.into();
    assert!(a.usable_by(Uuid::from_u128(7)));
    assert!(!a.usable_by(Uuid::from_u128(8)));

    a.status = STATUS_SUSPENDED.into();
    assert!(!a.usable_by(Uuid::from_u128(7)));
}

#[test]
fn pkce_s256_is_checked() {
    // Пара из RFC 7636, приложение B.
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    assert!(pkce_matches(challenge, Some("S256"), verifier));
    assert!(!pkce_matches(challenge, Some("S256"), "another-verifier"));
    // Метод по умолчанию — S256, а не plain: иначе challenge = verifier и
    // проверка ничего не значит.
    assert!(pkce_matches(challenge, None, verifier));
    assert!(!pkce_matches(verifier, None, verifier));
}

#[test]
fn unknown_pkce_methods_are_refused() {
    assert!(!pkce_matches("whatever", Some("md5"), "whatever"));
}
