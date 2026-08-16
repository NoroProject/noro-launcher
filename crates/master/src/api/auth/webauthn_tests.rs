//! Ошибка в rp_id не лечится: passkey привязывается к домену навсегда.

use super::*;

fn rp_id(web: &str, api: &str) -> Result<String> {
    rp_id_for(&Url::parse(web).unwrap(), &Url::parse(api).unwrap())
}

#[test]
fn api_on_a_subdomain_yields_the_site_domain() {
    assert_eq!(
        rp_id("https://example.dev", "https://api.example.dev").unwrap(),
        "example.dev"
    );
}

#[test]
fn same_host_on_different_ports_is_fine() {
    assert_eq!(
        rp_id("http://localhost:3000", "http://localhost:8080").unwrap(),
        "localhost"
    );
}

#[test]
fn site_on_a_subdomain_of_the_api_domain_works_too() {
    assert_eq!(
        rp_id("https://web.example.dev", "https://example.dev").unwrap(),
        "example.dev"
    );
}

#[test]
fn unrelated_domains_fail_at_startup_not_at_login() {
    assert!(rp_id("https://example.dev", "https://api.other.dev").is_err());
}

#[test]
fn a_suffix_that_is_not_a_subdomain_is_rejected() {
    // "notexample.dev" оканчивается на "example.dev" как строка, но доменом
    // ему не является — иначе чужой сайт получил бы наши ключи.
    assert!(rp_id("https://example.dev", "https://notexample.dev").is_err());
}
