//! Ошибка в приоритете правил либо стирает файлы игрока, либо навсегда
//! оставляет его на старом конфиге. Проверяются обе стороны.

use super::*;

fn rules(pairs: &[(&str, PathMode)]) -> Vec<PathRule> {
    pairs
        .iter()
        .map(|(pattern, mode)| PathRule {
            pattern: (*pattern).into(),
            mode: *mode,
            conflict: ConflictPolicy::default(),
        })
        .collect()
}

#[test]
fn the_last_matching_rule_wins() {
    // Ради этого список и стал упорядоченным: исключение пишется строкой ниже,
    // а не выносится в отдельный массив.
    let r = rules(&[
        ("config/**", PathMode::Merged),
        ("config/xaero*", PathMode::Unmanaged),
    ]);

    assert_eq!(mode_for("config/sodium.json", &r), PathMode::Merged);
    assert_eq!(mode_for("config/xaerominimap.txt", &r), PathMode::Unmanaged);
}

#[test]
fn an_unmatched_path_belongs_to_the_build() {
    // Умолчание — managed: файл, про который ничего не сказано, принадлежит
    // сборке. Обратное умолчание означало бы, что забытый путь молча перестаёт
    // обновляться.
    assert_eq!(mode_for("mods/core.jar", &rules(&[])), PathMode::Managed);
}

#[test]
fn a_directory_pattern_covers_everything_inside() {
    let r = rules(&[("saves/", PathMode::Unmanaged)]);

    assert_eq!(mode_for("saves/Мир/level.dat", &r), PathMode::Unmanaged);
    assert_eq!(mode_for("saves", &r), PathMode::Unmanaged);
    // Не должен цеплять соседа с тем же префиксом.
    assert_eq!(mode_for("saves_backup/x", &r), PathMode::Managed);
}

#[test]
fn a_prefix_pattern_matches_by_prefix() {
    let r = rules(&[("xaero*", PathMode::Unmanaged)]);

    assert_eq!(
        mode_for("XaeroWaypoints/Multiplayer_play/waypoints.txt", &r),
        PathMode::Unmanaged
    );
    assert_eq!(mode_for("mods/xaero.jar", &r), PathMode::Managed);
}

#[test]
fn an_exact_pattern_matches_only_itself() {
    let r = rules(&[("options.txt", PathMode::Merged)]);

    assert_eq!(mode_for("options.txt", &r), PathMode::Merged);
    assert_eq!(mode_for("optionsof.txt", &r), PathMode::Managed);
}

#[test]
fn matching_ignores_case() {
    // Windows отдаёт пути в том регистре, в котором их записали на диск.
    let r = rules(&[("Config/**", PathMode::Merged)]);
    assert_eq!(mode_for("config/sodium.json", &r), PathMode::Merged);
}

#[test]
fn legacy_lists_keep_their_precedence() {
    // `config/xaero*` из unmanaged обязан побеждать `config/` из user_managed:
    // иначе точки карты снова начнут перезаписываться.
    let converted = from_legacy(&["config/xaero*".into()], &["config/".into()]);

    assert_eq!(
        mode_for("config/sodium.json", &converted),
        PathMode::UserManaged
    );
    assert_eq!(
        mode_for("config/xaerominimap.txt", &converted),
        PathMode::Unmanaged
    );
}

#[test]
fn the_conflict_policy_comes_from_the_matching_rule() {
    let r = vec![
        PathRule {
            pattern: "config/**".into(),
            mode: PathMode::Merged,
            conflict: ConflictPolicy::KeepMine,
        },
        PathRule {
            pattern: "config/server.toml".into(),
            mode: PathMode::Merged,
            conflict: ConflictPolicy::TakeTheirs,
        },
    ];

    assert_eq!(
        rule_for("config/sodium.json", &r).unwrap().conflict,
        ConflictPolicy::KeepMine
    );
    assert_eq!(
        rule_for("config/server.toml", &r).unwrap().conflict,
        ConflictPolicy::TakeTheirs
    );
}
