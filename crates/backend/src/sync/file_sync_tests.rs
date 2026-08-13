//! Защита путей от удаления — ошибка здесь стирает данные игрока и заметна
//! только после того, как мир, скриншоты или настройки уже пропали.
//!
//! Маски `xaero*` и `config/xaero*` появились не на пустом месте: карта Xaero
//! хранит данные вне `saves/`, и синхронизация сносила их при каждом запуске.

use super::*;

fn protected_defaults() -> Vec<String> {
    [
        "saves/",
        "screenshots/",
        "options.txt",
        "optionsof.txt",
        "logs/",
        "crash-reports/",
        "xaero*",
        "config/xaero*",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[test]
fn directory_pattern_covers_everything_below_it() {
    let p = protected_defaults();
    assert!(is_protected("saves/world/level.dat", &p));
    assert!(is_protected("saves/world/region/r.0.0.mca", &p));
    assert!(is_protected("saves", &p), "сама директория тоже защищена");
    assert!(is_protected("logs/latest.log", &p));
}

#[test]
fn exact_pattern_matches_only_that_file() {
    let p = protected_defaults();
    assert!(is_protected("options.txt", &p));
    assert!(
        !is_protected("options.txt.bak", &p),
        "точный шаблон не должен цеплять соседние имена"
    );
    assert!(!is_protected("config/options.txt", &p));
}

/// Ровно тот случай, ради которого заводили маски: данные Xaero лежат в корне
/// инстанса и в `config/`, а не в `saves/`.
#[test]
fn wildcard_protects_xaero_data() {
    let p = protected_defaults();
    assert!(is_protected("xaerominimap/waypoints.txt", &p));
    assert!(is_protected("xaeroworldmap/1.21/region.zip", &p));
    assert!(is_protected("config/xaeroworldmap.txt", &p));
    assert!(is_protected("config/xaerominimap.txt", &p));
}

/// Маска у корня не должна распространяться на вложенные каталоги сама по
/// себе — поэтому `config/xaero*` и заведён отдельной строкой.
#[test]
fn root_wildcard_does_not_reach_into_subdirectories() {
    let only_root = vec!["xaero*".to_string()];
    assert!(is_protected("xaerominimap/waypoints.txt", &only_root));
    assert!(!is_protected("config/xaeroworldmap.txt", &only_root));
}

#[test]
fn managed_files_stay_deletable() {
    let p = protected_defaults();
    assert!(!is_protected("mods/jei.jar", &p));
    assert!(!is_protected("config/jei/settings.ini", &p));
    assert!(!is_protected("versions/1.21.1/client.jar", &p));
}

/// Windows отдаёт пути в другом регистре, а манифест пишет админ руками.
#[test]
fn matching_ignores_case() {
    let p = protected_defaults();
    assert!(is_protected("Saves/World/level.dat", &p));
    assert!(is_protected("OPTIONS.TXT", &p));
    assert!(is_protected("XaeroMinimap/waypoints.txt", &p));
}

#[test]
fn empty_pattern_list_protects_nothing() {
    assert!(!is_protected("saves/world/level.dat", &[]));
}
