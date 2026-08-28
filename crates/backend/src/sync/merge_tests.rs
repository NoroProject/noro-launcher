//! The decision table, cell by cell. Getting it wrong either overwrites the
//! player's edits or leaves them on an old config forever.

use super::*;

const MINE: &str = "aaa";
const BASE: &str = "bbb";
const THEIRS: &str = "ccc";

#[test]
fn untouched_by_the_player_gets_updated() {
    // Matches the base and the server moved on. The case the mode exists for:
    // `user_managed` would leave the player on the old file forever.
    assert_eq!(decide(Some(BASE), Some(BASE), THEIRS), Decision::Update);
}

#[test]
fn edited_by_the_player_and_untouched_by_the_server_stays() {
    assert_eq!(decide(Some(MINE), Some(BASE), BASE), Decision::KeepMine);
}

#[test]
fn edited_by_both_is_a_conflict() {
    assert_eq!(decide(Some(MINE), Some(BASE), THEIRS), Decision::Conflict);
}

#[test]
fn nothing_changed_means_nothing_to_do() {
    assert_eq!(decide(Some(BASE), Some(BASE), BASE), Decision::Nothing);
}

#[test]
fn identical_edits_are_not_a_conflict() {
    // Both sides landed on the same contents; nothing to argue about.
    assert_eq!(decide(Some(THEIRS), Some(BASE), THEIRS), Decision::Nothing);
}

#[test]
fn a_missing_file_is_always_installed() {
    assert_eq!(decide(None, Some(BASE), THEIRS), Decision::Update);
    assert_eq!(decide(None, None, THEIRS), Decision::Update);
}

#[test]
fn without_a_base_a_difference_is_treated_as_a_conflict() {
    // First pass over an already installed build: there is no base, and a file
    // must not be overwritten just because we don't remember it.
    assert_eq!(decide(Some(MINE), None, THEIRS), Decision::Conflict);
    assert_eq!(decide(Some(THEIRS), None, THEIRS), Decision::Nothing);
}

#[tokio::test]
async fn base_hashes_survive_a_round_trip() {
    let dir = std::env::temp_dir().join(format!("noro-merge-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let mut base = BaseHashes::default();
    base.set("config/sodium.json", BASE);
    base.save(&dir).await;

    let loaded = BaseHashes::load(&dir).await;
    assert_eq!(loaded.get("config/sodium.json"), Some(BASE));
    assert_eq!(loaded.get("config/other.json"), None);

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn a_conflict_backup_keeps_the_players_version() {
    let dir = std::env::temp_dir().join(format!("noro-conflict-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("config")).unwrap();
    std::fs::write(dir.join("config/sodium.json"), "the player's edits").unwrap();

    backup_conflict(&dir, "config/sodium.json", "20260816-120000")
        .await
        .unwrap();

    let saved =
        std::fs::read_to_string(dir.join(".noro/conflicts/20260816-120000/config/sodium.json"))
            .unwrap();
    assert_eq!(saved, "the player's edits");

    let _ = std::fs::remove_dir_all(&dir);
}
