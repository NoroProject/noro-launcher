//! A wrong rp_id can't be fixed later: a passkey is bound to its domain for good.

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
    // "notexample.dev" ends with "example.dev" as a string but isn't a
    // subdomain of it; otherwise someone else's site would get our keys.
    assert!(rp_id("https://example.dev", "https://notexample.dev").is_err());
}
