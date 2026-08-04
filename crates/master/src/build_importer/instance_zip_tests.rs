use super::*;

fn paths(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn unwraps_zipped_folder() {
    let p = paths(&["MyPack/mods/a.jar", "MyPack/config/b.toml"]);
    assert_eq!(wrapper_prefix(&p).as_deref(), Some("MyPack/"));
}

#[test]
fn keeps_root_when_archive_is_the_root() {
    let p = paths(&["mods/a.jar", "config/b.toml", "options.txt"]);
    assert_eq!(wrapper_prefix(&p), None);
}

/// Единственная папка верхнего уровня — ещё не обёртка: архив только из модов
/// развернулся бы в голые jar-файлы без `mods/`.
#[test]
fn mods_only_archive_is_not_a_wrapper() {
    let p = paths(&["mods/a.jar", "mods/b.jar"]);
    assert_eq!(wrapper_prefix(&p), None);
}

#[test]
fn mixed_top_levels_are_not_a_wrapper() {
    let p = paths(&["MyPack/mods/a.jar", "Other/config/b.toml"]);
    assert_eq!(wrapper_prefix(&p), None);
}

#[test]
fn file_at_root_blocks_unwrapping() {
    let p = paths(&["MyPack/mods/a.jar", "readme.txt"]);
    assert_eq!(wrapper_prefix(&p), None);
}

#[test]
fn rejects_paths_escaping_the_build_root() {
    assert_eq!(safe_path("../evil.jar"), None);
    assert_eq!(safe_path("mods/../../evil.jar"), None);
    assert_eq!(safe_path("/etc/passwd"), None);
    assert_eq!(safe_path("C:\\Windows\\x.dll"), None);
}

#[test]
fn normalizes_windows_separators() {
    assert_eq!(safe_path("mods\\a.jar").as_deref(), Some("mods/a.jar"));
    assert_eq!(
        safe_path("./config/b.toml").as_deref(),
        Some("config/b.toml")
    );
}

#[test]
fn skips_archiver_junk() {
    assert!(is_junk("__MACOSX/mods/._a.jar"));
    assert!(is_junk("config/.DS_Store"));
    assert!(!is_junk("mods/a.jar"));
}
