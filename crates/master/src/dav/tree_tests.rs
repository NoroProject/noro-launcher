use super::*;
use uuid::Uuid;

fn row(path: &str, size: i64) -> BuildFileRow {
    BuildFileRow {
        id: Uuid::new_v4(),
        build_id: Uuid::new_v4(),
        path: path.into(),
        sha1: "da39a3ee".into(),
        size,
        side: "both".into(),
        kind: "config".into(),
    }
}

fn sample() -> Vec<BuildFileRow> {
    vec![
        row("mods/create.jar", 100),
        row("config/create/common.toml", 10),
        row("config/kubejs/server.json", 20),
        row("options.txt", 5),
    ]
}

#[test]
fn root_lists_folders_and_files() {
    let files = sample();
    let names: Vec<_> = children(&files, "")
        .into_iter()
        .map(|n| (n.name, n.is_dir))
        .collect();

    assert!(names.contains(&("mods".to_string(), true)));
    assert!(names.contains(&("config".to_string(), true)));
    assert!(names.contains(&("options.txt".to_string(), false)));
    // `config` встречается дважды в путях, но в списке должен быть один раз.
    assert_eq!(names.iter().filter(|(n, _)| n == "config").count(), 1);
}

#[test]
fn nested_folder_lists_only_its_own_level() {
    let files = sample();
    let names: Vec<_> = children(&files, "config")
        .into_iter()
        .map(|n| (n.name, n.is_dir))
        .collect();

    assert_eq!(
        names,
        vec![("create".to_string(), true), ("kubejs".to_string(), true)]
    );
}

#[test]
fn directories_exist_only_while_they_hold_files() {
    let files = sample();
    assert!(is_dir(&files, "config"));
    assert!(is_dir(&files, "config/create"));
    assert!(!is_dir(&files, "config/nope"));
    // Файл каталогом не считается.
    assert!(!is_dir(&files, "options.txt"));
}

#[test]
fn under_collects_whole_subtree() {
    let files = sample();
    let paths: Vec<_> = under(&files, "config").iter().map(|f| f.path.clone()).collect();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().all(|p| p.starts_with("config/")));
}

#[test]
fn normalize_rejects_escaping_and_trims_slashes() {
    assert_eq!(normalize("/config/foo.toml/").as_deref(), Some("config/foo.toml"));
    assert_eq!(normalize("").as_deref(), Some(""));
    assert_eq!(normalize("//config//a//"). as_deref(), Some("config/a"));
    // Выход за корень сборки недопустим.
    assert_eq!(normalize("../../etc/passwd"), None);
    assert_eq!(normalize("config/../../x"), None);
}

#[test]
fn macos_junk_is_recognised_by_any_segment() {
    assert!(is_macos_junk("mods/._NBitChat.jar"));
    assert!(is_macos_junk(".DS_Store"));
    assert!(is_macos_junk(".Trashes/501/whatever.jar"));
    // Точка в имени сама по себе мусором не делает.
    assert!(!is_macos_junk("mods/NBitChat.jar"));
    assert!(!is_macos_junk("config/.mixin.out/x"));
}
