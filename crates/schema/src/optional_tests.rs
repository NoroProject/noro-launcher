use super::*;

fn opt(name: &str, deps: &[&str], conflicts: &[&str]) -> OptionalMod {
    OptionalMod {
        name: name.into(),
        description: String::new(),
        category: String::new(),
        files: vec![],
        enabled_by_default: false,
        visible: true,
        limited: false,
        dependencies: deps.iter().map(|s| s.to_string()).collect(),
        conflicts: conflicts.iter().map(|s| s.to_string()).collect(),
        triggers: vec![],
        os: vec![],
        icon_url: None,
        author: None,
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn free_choice_is_allowed() {
    let mods = [opt("sodium", &[], &[]), opt("iris", &[], &[])];
    assert!(can_enable(&mods, &names(&["sodium"]), "iris").is_ok());
}

#[test]
fn conflict_blocks_enabling() {
    let mods = [opt("sodium", &[], &["optifine"]), opt("optifine", &[], &[])];

    let issue = can_enable(&mods, &names(&["optifine"]), "sodium").unwrap_err();

    assert_eq!(
        issue,
        SelectionIssue::Conflict {
            mod_name: "sodium".into(),
            with: "optifine".into()
        }
    );
}

/// Несовместимость записывают у одного из пары, а мешает она обоим: иначе
/// порядок включения решал бы, поймаем мы конфликт или нет.
#[test]
fn conflict_works_from_either_side() {
    let mods = [opt("sodium", &[], &["optifine"]), opt("optifine", &[], &[])];

    assert!(can_enable(&mods, &names(&["sodium"]), "optifine").is_err());
}

#[test]
fn dependency_must_be_enabled_first() {
    let mods = [opt("iris", &["sodium"], &[]), opt("sodium", &[], &[])];

    let issue = can_enable(&mods, &names(&[]), "iris").unwrap_err();
    assert_eq!(
        issue,
        SelectionIssue::MissingDependency {
            mod_name: "iris".into(),
            needs: "sodium".into()
        }
    );
    assert!(can_enable(&mods, &names(&["sodium"]), "iris").is_ok());
}

/// Мода нет в сборке — запрещать нечего: список мог обновиться, а старый выбор
/// игрока это не повод не пустить его в игру.
#[test]
fn unknown_mod_is_not_blocked() {
    assert!(can_enable(&[], &names(&["whatever"]), "ghost").is_ok());
}

/// Готовый набор проверяется целиком: сборку могли изменить после того, как
/// игрок его собрал.
#[test]
fn broken_selection_is_reported_once_per_pair() {
    let mods = [
        opt("sodium", &[], &["optifine"]),
        opt("optifine", &[], &["sodium"]),
        opt("iris", &["sodium"], &[]),
    ];

    let found = issues(&mods, &names(&["sodium", "optifine", "iris"]));

    // Пара несовместимых даёт одну запись, а не две зеркальные.
    let conflicts = found
        .iter()
        .filter(|i| matches!(i, SelectionIssue::Conflict { .. }))
        .count();
    assert_eq!(conflicts, 1);
    assert!(!found.is_empty());
}

#[test]
fn complete_selection_has_no_issues() {
    let mods = [opt("iris", &["sodium"], &[]), opt("sodium", &[], &[])];

    assert!(issues(&mods, &names(&["sodium", "iris"])).is_empty());
}
