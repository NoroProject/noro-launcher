use super::*;
use axum::http::Method;

#[test]
fn reads_are_not_recorded() {
    // Под impersonation читающих запросов сотни на каждый экран, а спор
    // разрешают изменения.
    assert!(!is_mutating(&Method::GET));
    assert!(!is_mutating(&Method::HEAD));
    assert!(!is_mutating(&Method::OPTIONS));
}

#[test]
fn every_change_is_recorded() {
    for m in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
        assert!(is_mutating(&m), "{m} должен писаться в аудит");
    }
}
