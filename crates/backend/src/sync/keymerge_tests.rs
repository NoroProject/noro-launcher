//! The whole point is that edits to different keys coexist. Getting it wrong
//! either loses a player's setting or silently merges what shouldn't be merged.

use super::*;

fn lines(text: &str) -> Vec<&str> {
    text.lines().collect()
}

#[test]
fn edits_to_different_keys_live_together() {
    // The case key-level merging exists for: a whole-file three-way would call
    // this a conflict.
    let base = "fov=70\nrender=12\nsound=1.0";
    let mine = "fov=90\nrender=12\nsound=1.0";
    let theirs = "fov=70\nrender=16\nsound=1.0";

    let out = merge_properties(mine, base, theirs).unwrap();

    assert_eq!(lines(&out), ["fov=90", "render=16", "sound=1.0"]);
}

#[test]
fn the_same_key_changed_differently_is_left_to_the_human() {
    let base = "fov=70";
    assert!(merge_properties("fov=90", base, "fov=100").is_none());
}

#[test]
fn identical_changes_are_not_a_conflict() {
    let out = merge_properties("fov=90", "fov=70", "fov=90").unwrap();
    assert_eq!(lines(&out), ["fov=90"]);
}

#[test]
fn a_key_added_by_the_server_arrives() {
    let out = merge_properties("fov=70", "fov=70", "fov=70\nnewOption=true").unwrap();
    assert_eq!(lines(&out), ["fov=70", "newOption=true"]);
}

#[test]
fn a_key_added_by_the_player_survives() {
    let out = merge_properties("fov=70\nmyOption=1", "fov=70", "fov=70").unwrap();
    assert_eq!(lines(&out), ["fov=70", "myOption=1"]);
}

#[test]
fn a_key_the_player_deleted_stays_deleted() {
    // A deletion is an edit; an update must not undo it.
    let out = merge_properties("render=12", "fov=70\nrender=12", "fov=70\nrender=12").unwrap();
    assert_eq!(lines(&out), ["render=12"]);
}

#[test]
fn a_key_the_server_removed_goes_away() {
    let out = merge_properties("fov=70\nold=1", "fov=70\nold=1", "fov=70").unwrap();
    assert_eq!(lines(&out), ["fov=70"]);
}

#[test]
fn colon_separated_lines_are_understood() {
    // Minecraft's options.txt uses a colon.
    let out = merge_properties("fov:90", "fov:70", "fov:70").unwrap();
    assert_eq!(lines(&out), ["fov=90"]);
}

#[test]
fn comments_and_blank_lines_do_not_break_parsing() {
    let base = "# a comment\n\nfov=70";
    let out = merge_properties("# a different one\nfov=90", base, base).unwrap();
    assert_eq!(lines(&out), ["fov=90"]);
}

#[test]
fn only_known_formats_are_offered_for_merging() {
    assert!(is_mergeable("config/mod.properties"));
    assert!(is_mergeable("options.txt"));
    // JSON and TOML are left out on purpose: a value there can be a tree, and
    // merging by key stops being well-defined.
    assert!(!is_mergeable("config/sodium.json"));
    assert!(!is_mergeable("config/server.toml"));
    assert!(!is_mergeable("mods/core.jar"));
}

mod network {
    use super::super::{base_copy_path, try_merge};
    use crate::test_http::{serve, sha1_hex, TempDir};

    async fn instance(base: &str, mine: &str) -> TempDir {
        let dir = TempDir::new("keymerge");
        let base_path = base_copy_path(dir.path(), "options.txt");
        tokio::fs::create_dir_all(base_path.parent().unwrap())
            .await
            .unwrap();
        tokio::fs::write(&base_path, base).await.unwrap();
        tokio::fs::write(dir.path().join("options.txt"), mine)
            .await
            .unwrap();
        dir
    }

    #[tokio::test]
    async fn merges_and_moves_the_base_to_the_server_copy() {
        let theirs = "fov:70\nrender:16";
        let server = serve(vec![("/o", 200, theirs.as_bytes().to_vec())]).await;
        let dir = instance("fov:70\nrender:12", "fov:90\nrender:12").await;

        let merged = try_merge(
            &reqwest::Client::new(),
            dir.path(),
            "options.txt",
            &server.url("/o"),
            &sha1_hex(theirs.as_bytes()),
        )
        .await
        .expect("different keys merge");

        assert_eq!(merged, "fov=90\nrender=16");
        let on_disk = tokio::fs::read_to_string(dir.path().join("options.txt"))
            .await
            .unwrap();
        assert_eq!(on_disk, merged);
        let base = tokio::fs::read_to_string(base_copy_path(dir.path(), "options.txt"))
            .await
            .unwrap();
        assert_eq!(base, theirs, "the base is the server's text, not the merge");
    }

    #[tokio::test]
    async fn a_body_that_does_not_match_the_manifest_is_not_merged() {
        let server = serve(vec![("/o", 200, b"<html>gateway error</html>".to_vec())]).await;
        let dir = instance("fov:70", "fov:90").await;

        let merged = try_merge(
            &reqwest::Client::new(),
            dir.path(),
            "options.txt",
            &server.url("/o"),
            &sha1_hex(b"fov:70\nrender:16"),
        )
        .await;

        assert!(merged.is_none());
        let on_disk = tokio::fs::read_to_string(dir.path().join("options.txt"))
            .await
            .unwrap();
        assert_eq!(on_disk, "fov:90", "the player's file is untouched");
    }

    #[tokio::test]
    async fn an_error_status_is_not_merged() {
        let body = b"fov:70".to_vec();
        let sha = sha1_hex(&body);
        let server = serve(vec![("/o", 503, body)]).await;
        let dir = instance("fov:70", "fov:90").await;

        let merged = try_merge(
            &reqwest::Client::new(),
            dir.path(),
            "options.txt",
            &server.url("/o"),
            &sha,
        )
        .await;
        assert!(merged.is_none());
    }
}
