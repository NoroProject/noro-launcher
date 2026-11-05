use super::*;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("noro-shots-{name}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_missing_folder_is_an_empty_gallery() {
    let dir = scratch("missing").join("screenshots");
    assert!(list(&dir).is_empty());
}

#[test]
fn only_pictures_are_listed() {
    let dir = scratch("filter");
    std::fs::write(dir.join("2026-11-05_01.00.00.png"), b"x").unwrap();
    std::fs::write(dir.join("notes.txt"), b"x").unwrap();
    std::fs::create_dir(dir.join("folder.png")).unwrap();

    let names: Vec<_> = list(&dir).into_iter().map(|s| s.file_name).collect();
    assert_eq!(names, vec!["2026-11-05_01.00.00.png".to_string()]);
}

#[test]
fn newest_first() {
    let dir = scratch("order");
    let base = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
    for (name, age) in [("old.png", 0), ("new.png", 200), ("mid.png", 100)] {
        let path = dir.join(name);
        std::fs::write(&path, b"x").unwrap();
        let file = std::fs::File::options().write(true).open(&path).unwrap();
        file.set_modified(base + std::time::Duration::from_secs(age))
            .unwrap();
    }
    let names: Vec<_> = list(&dir).into_iter().map(|s| s.file_name).collect();
    assert_eq!(names, vec!["new.png", "mid.png", "old.png"]);
}

#[test]
fn resolve_stays_inside_the_folder() {
    let root = scratch("escape");
    let dir = root.join("screenshots");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.png"), b"x").unwrap();
    std::fs::write(root.join("secret.png"), b"x").unwrap();

    assert_eq!(resolve(&dir, "a.png"), Some(dir.join("a.png")));
    assert_eq!(resolve(&dir, "../secret.png"), None);
    assert_eq!(resolve(&dir, "..\\secret.png"), None);
    assert_eq!(resolve(&dir, "sub/a.png"), None);
    assert_eq!(resolve(&dir, ".."), None);
    assert_eq!(resolve(&dir, ""), None);
    assert_eq!(resolve(&dir, "missing.png"), None);
}

#[cfg(unix)]
#[test]
fn links_are_neither_listed_nor_resolved() {
    let root = scratch("link");
    let dir = root.join("screenshots");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(root.join("outside.png"), b"x").unwrap();
    std::os::unix::fs::symlink(root.join("outside.png"), dir.join("link.png")).unwrap();

    assert!(list(&dir).is_empty());
    assert_eq!(resolve(&dir, "link.png"), None);
}
