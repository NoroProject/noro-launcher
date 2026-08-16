//! Ложное срабатывание здесь удаляет файл игрока, пропуск — оставляет xray.
//! Проверяются обе стороны.

use super::*;

fn by_name(pattern: &str) -> BlockedFile {
    BlockedFile {
        pattern: Some(pattern.into()),
        sha1: None,
        reason: "тест".into(),
        action: BlockAction::Delete,
    }
}

fn by_hash(sha1: &str) -> BlockedFile {
    BlockedFile {
        pattern: None,
        sha1: Some(sha1.into()),
        reason: "тест".into(),
        action: BlockAction::Delete,
    }
}

const SHA: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";

#[test]
fn a_name_mask_catches_the_obvious() {
    let r = by_name("*xray*");
    assert!(r.matches("resourcepacks/super-xray-1.20.zip", SHA));
    assert!(r.matches("mods/XRay.jar", SHA));
    assert!(!r.matches("mods/sodium.jar", SHA));
}

#[test]
fn a_hash_rule_ignores_the_name() {
    // Переименование — самый дешёвый обход маски, и хеш его переживает.
    let r = by_hash(SHA);
    assert!(r.matches("resourcepacks/безобидное-имя.zip", SHA));
    assert!(!r.matches("resourcepacks/безобидное-имя.zip", "0".repeat(40).as_str()));
}

#[test]
fn a_hash_comparison_ignores_case() {
    assert!(by_hash(&SHA.to_uppercase()).matches("x.zip", SHA));
}

#[test]
fn both_conditions_must_hold_when_both_are_set() {
    // «Этот хеш под этим именем» строже каждого по отдельности и даёт меньше
    // ложных срабатываний.
    let r = BlockedFile {
        pattern: Some("mods/*.jar".into()),
        sha1: Some(SHA.into()),
        reason: "тест".into(),
        action: BlockAction::Delete,
    };

    assert!(r.matches("mods/cheat.jar", SHA));
    assert!(!r.matches("mods/cheat.jar", "0".repeat(40).as_str()));
    assert!(!r.matches("resourcepacks/cheat.zip", SHA));
}

#[test]
fn an_empty_rule_matches_nothing() {
    // Иначе пустая строка в админке удалила бы игроку весь каталог.
    let r = BlockedFile {
        pattern: None,
        sha1: None,
        reason: "пусто".into(),
        action: BlockAction::Delete,
    };
    assert!(!r.matches("mods/anything.jar", SHA));
}

#[test]
fn a_mask_anchored_at_the_end_does_not_match_the_middle() {
    let r = by_name("mods/*.jar");
    assert!(r.matches("mods/sodium.jar", SHA));
    assert!(!r.matches("mods/sodium.jar.disabled", SHA));
}

#[test]
fn a_mask_without_stars_is_an_exact_path() {
    let r = by_name("mods/banned.jar");
    assert!(r.matches("mods/banned.jar", SHA));
    assert!(!r.matches("mods/banned.jar.bak", SHA));
    assert!(!r.matches("other/mods/banned.jar", SHA));
}

#[test]
fn the_first_matching_rule_decides() {
    // Порядок задаёт админ, и более узкое правило ставится выше.
    let rules = vec![
        BlockedFile {
            pattern: Some("*xray*".into()),
            sha1: None,
            reason: "известная сборка".into(),
            action: BlockAction::BlockLaunch,
        },
        by_name("*"),
    ];

    let hit = first_match(&rules, "mods/xray.jar", SHA).unwrap();
    assert_eq!(hit.action, BlockAction::BlockLaunch);
    assert_eq!(hit.reason, "известная сборка");
}

#[test]
fn nothing_matches_an_ordinary_file() {
    let rules = vec![by_name("*xray*"), by_hash(SHA)];
    assert!(first_match(&rules, "mods/sodium.jar", &"1".repeat(40)).is_none());
}
