//! Смысл слияния — в том, что правки разных ключей уживаются. Ошибка здесь
//! либо теряет настройку игрока, либо молча склеивает несклеиваемое.

use super::*;

fn lines(text: &str) -> Vec<&str> {
    text.lines().collect()
}

#[test]
fn edits_to_different_keys_live_together() {
    // Ровно тот случай, ради которого key-level и нужен: файловый three-way
    // назвал бы это конфликтом.
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
    // Удаление — тоже правка, и затирать её обновлением нельзя.
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
    // options.txt Minecraft использует двоеточие.
    let out = merge_properties("fov:90", "fov:70", "fov:70").unwrap();
    assert_eq!(lines(&out), ["fov=90"]);
}

#[test]
fn comments_and_blank_lines_do_not_break_parsing() {
    let base = "# комментарий\n\nfov=70";
    let out = merge_properties("# другой\nfov=90", base, base).unwrap();
    assert_eq!(lines(&out), ["fov=90"]);
}

#[test]
fn only_known_formats_are_offered_for_merging() {
    assert!(is_mergeable("config/mod.properties"));
    assert!(is_mergeable("options.txt"));
    // JSON и TOML намеренно не поддерживаются: там значение бывает деревом, и
    // «слияние по ключам» перестаёт быть однозначным.
    assert!(!is_mergeable("config/sodium.json"));
    assert!(!is_mergeable("config/server.toml"));
    assert!(!is_mergeable("mods/core.jar"));
}
