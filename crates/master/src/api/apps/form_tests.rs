//! Проверка формы приложения. Адрес возврата — это то место, куда уедет код
//! доступа к чужому аккаунту, поэтому проверяется он, а не только длина имени.

use super::*;

fn form(name: &str, uris: &str) -> AppForm {
    AppForm {
        name: name.into(),
        description: None,
        redirect_uris: uris.into(),
        scopes: vec![schema::SCOPE_IDENTITY.into()],
    }
}

#[test]
fn a_normal_form_passes() {
    let (name, desc, uris) = form("My App", "https://app.example/callback")
        .check()
        .unwrap();
    assert_eq!(name, "My App");
    assert_eq!(desc, None);
    assert_eq!(uris, "https://app.example/callback");
}

#[test]
fn several_redirects_are_kept_line_by_line() {
    let (_, _, uris) = form(
        "App",
        "https://app.example/cb\n  http://localhost:3000/cb  \n\n",
    )
    .check()
    .unwrap();
    assert_eq!(uris.lines().count(), 2);
}

#[test]
fn a_form_without_redirects_is_refused() {
    assert!(form("App", "   \n\n").check().is_err());
}

#[test]
fn javascript_urls_are_refused() {
    assert!(form("App", "javascript:alert(1)").check().is_err());
}

#[test]
fn fragments_are_refused() {
    assert!(form("App", "https://app.example/cb#token").check().is_err());
}

#[test]
fn a_nameless_app_is_refused() {
    assert!(form("   ", "https://app.example/cb").check().is_err());
}

#[test]
fn an_overlong_description_is_refused() {
    let mut f = form("App", "https://app.example/cb");
    f.description = Some("x".repeat(501));
    assert!(f.check().is_err());
}

#[test]
fn an_owner_cannot_grant_himself_a_privileged_scope() {
    let mut f = form("App", "https://app.example/cb");
    f.scopes = vec![schema::SCOPE_IDENTITY.into(), schema::SCOPE_JOURNAL.into()];
    assert!(f.check_scopes().is_err());
}

#[test]
fn an_empty_choice_still_means_identity() {
    let mut f = form("App", "https://app.example/cb");
    f.scopes = Vec::new();
    assert_eq!(f.check_scopes().unwrap(), [schema::SCOPE_IDENTITY]);
}

#[test]
fn duplicates_collapse() {
    let mut f = form("App", "https://app.example/cb");
    f.scopes = vec![schema::SCOPE_SKINS.into(), schema::SCOPE_SKINS.into()];
    assert_eq!(f.check_scopes().unwrap(), [schema::SCOPE_SKINS]);
}
