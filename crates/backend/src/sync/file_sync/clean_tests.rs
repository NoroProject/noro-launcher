//! A mistake in path protection deletes the player's data, and only shows up
//! once the world, the screenshots or the settings are already gone.

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
    assert!(is_protected("saves", &p), "the directory itself counts too");
    assert!(is_protected("logs/latest.log", &p));
}

#[test]
fn exact_pattern_matches_only_that_file() {
    let p = protected_defaults();
    assert!(is_protected("options.txt", &p));
    assert!(
        !is_protected("options.txt.bak", &p),
        "an exact pattern must not catch neighbouring names"
    );
    assert!(!is_protected("config/options.txt", &p));
}

/// The case the wildcards exist for: Xaero's map keeps its data at the instance
/// root and in `config/`, not under `saves/`.
#[test]
fn wildcard_protects_xaero_data() {
    let p = protected_defaults();
    assert!(is_protected("xaerominimap/waypoints.txt", &p));
    assert!(is_protected("xaeroworldmap/1.21/region.zip", &p));
    assert!(is_protected("config/xaeroworldmap.txt", &p));
    assert!(is_protected("config/xaerominimap.txt", &p));
}

/// A root wildcard doesn't reach into subdirectories on its own, which is why
/// `config/xaero*` needs its own entry.
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

/// Windows hands back paths in a different case, and the manifest is written by
/// hand.
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
