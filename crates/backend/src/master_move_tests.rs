use super::*;
use crate::test_http::TempDir;

const OLD: &str = "https://api.old.example.com";
const NEW: &str = "https://api.new.example.com";

fn url(s: &str) -> Url {
    Url::parse(s).unwrap()
}

fn from_old(header: Option<&str>) -> Option<String> {
    announced(
        OLD,
        &url("https://api.old.example.com/api/launcher/version"),
        header,
    )
}

#[test]
fn an_https_origin_from_the_current_master_is_a_move() {
    assert_eq!(from_old(Some(NEW)).as_deref(), Some(NEW));
    assert_eq!(
        from_old(Some("https://API.New.example.com/")).as_deref(),
        Some(NEW)
    );
    assert_eq!(
        from_old(Some(" https://api.new.example.com:8443 ")).as_deref(),
        Some("https://api.new.example.com:8443")
    );
}

#[test]
fn the_same_address_is_no_move() {
    assert_eq!(from_old(None), None);
    assert_eq!(from_old(Some(OLD)), None);
    assert_eq!(from_old(Some("https://API.old.example.com/")), None);
    assert_eq!(
        announced("https://api.old.example.com/", &url(OLD), Some(OLD)),
        None
    );
}

#[test]
fn a_header_that_cant_be_trusted_is_ignored() {
    for bad in [
        "",
        "   ",
        "not a url",
        "api.new.example.com",
        // Never from TLS down to plain text.
        "http://api.new.example.com",
        "ws://api.new.example.com",
        "file:///etc/passwd",
        // Origin only: core joins the paths itself.
        "https://api.new.example.com/master",
        "https://api.new.example.com/?next=1",
        "https://api.new.example.com/#top",
        "https://user:pw@api.new.example.com",
    ] {
        assert_eq!(
            from_old(Some(bad)),
            None,
            "{bad:?} must not move the launcher"
        );
    }
}

#[test]
fn only_an_answer_from_the_current_master_over_tls_counts() {
    // Followed a redirect: whoever answered is not the master core asked.
    let elsewhere = url("https://cdn.example.com/api/launcher/version");
    assert_eq!(announced(OLD, &elsewhere, Some(NEW)), None);
    // Plain http from a remote host: anyone on the path could have written it.
    let plain = url("http://api.old.example.com/api/launcher/version");
    assert_eq!(
        announced("http://api.old.example.com", &plain, Some(NEW)),
        None
    );
    // No address of our own to compare with.
    assert_eq!(announced("", &url(OLD), Some(NEW)), None);
}

#[cfg(debug_assertions)]
#[test]
fn a_debug_build_hears_a_local_master_over_http_but_still_moves_to_https() {
    let local = url("http://127.0.0.1:8080/api/launcher/version");
    assert_eq!(
        announced("http://127.0.0.1:8080", &local, Some(NEW)).as_deref(),
        Some(NEW)
    );
    assert_eq!(
        announced(
            "http://127.0.0.1:8080",
            &local,
            Some("http://localhost:8080")
        ),
        None
    );
}

fn moved_config() -> LauncherConfig {
    let mut c = LauncherConfig {
        master_url: format!("{OLD}/"),
        ..LauncherConfig::default()
    };
    assert_eq!(c.move_master(NEW), format!("{OLD}/"));
    c
}

#[test]
fn a_move_remembers_the_old_origin() {
    let c = moved_config();
    assert_eq!(c.master_url, NEW);
    assert_eq!(c.master_moved_from, vec![OLD.to_string()]);
}

/// The bootstrapper installed before the move hands the old address over on
/// every start. Taking it would undo the move each time.
#[test]
fn an_old_bootstrapper_does_not_roll_the_move_back() {
    let mut c = moved_config();
    assert_eq!(c.adopt_master(OLD.to_string()), None);
    assert_eq!(c.adopt_master(format!("{OLD}/")), None);
    assert_eq!(c.master_url, NEW);
}

/// A launcher reinstalled from a build stamped elsewhere still goes there.
#[test]
fn a_bootstrapper_stamped_with_another_address_still_wins() {
    let mut c = moved_config();
    let third = "https://api.third.example.com";
    assert_eq!(c.adopt_master(third.to_string()).as_deref(), Some(NEW));
    assert_eq!(c.master_url, third);
}

#[test]
fn moving_back_makes_the_address_current_again() {
    let mut c = moved_config();
    c.move_master(OLD);
    assert_eq!(c.master_moved_from, vec![NEW.to_string()]);
    assert_eq!(c.adopt_master(OLD.to_string()), None);
    assert_eq!(c.master_url, OLD);
}

#[test]
fn bootstrap_json_keeps_its_other_fields() {
    let dir = TempDir::new("master-move");
    let path = dir.path().join("bootstrap.json");
    std::fs::write(
        &path,
        r#"{"master_url":"https://api.old.example.com","signing_pubkey":"abcd"}"#,
    )
    .unwrap();
    let moved_from = vec![OLD.to_string()];

    assert!(write_bootstrap(&path, NEW, &moved_from).unwrap());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(v["master_url"], NEW);
    assert_eq!(v["moved_from"], serde_json::json!([OLD]));
    assert_eq!(v["signing_pubkey"], "abcd");

    // Already says so: no write on every start.
    assert!(!write_bootstrap(&path, NEW, &moved_from).unwrap());
}

#[test]
fn a_missing_or_broken_bootstrap_json_is_written_fresh() {
    let dir = TempDir::new("master-move-broken");
    let path = dir.path().join("bootstrap.json");
    let moved_from = vec![OLD.to_string()];
    assert!(write_bootstrap(&path, NEW, &moved_from).unwrap());
    std::fs::write(&path, "[not an object").unwrap();
    assert!(write_bootstrap(&path, NEW, &moved_from).unwrap());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(v["master_url"], NEW);
}

#[test]
fn nothing_is_written_without_a_move() {
    let dir = TempDir::new("master-move-none");
    let path = dir.path().join("bootstrap.json");
    sync_bootstrap(&path, NEW, &[]);
    assert!(!path.exists());
}
