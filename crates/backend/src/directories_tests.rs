use super::*;
use crate::test_http::TempDir;

#[test]
fn a_fresh_install_keeps_everything_in_one_folder() {
    let dirs = LauncherDirectories::at(PathBuf::from("/svc"));
    assert!(dirs.data_root_is_default());
    assert_eq!(dirs.instances(), Path::new("/svc/instances"));
    assert_eq!(dirs.store(), Path::new("/svc/store"));
    assert!(dirs.data_available());
}

#[test]
fn only_instances_and_the_store_follow_the_data_root() {
    let dirs = LauncherDirectories::at(PathBuf::from("/svc"));
    dirs.set_data_root(Some(Path::new("/disk/noro")));
    let id = uuid::Uuid::nil();

    for heavy in [
        dirs.instances(),
        dirs.instance(&id),
        dirs.natives(&id),
        dirs.store(),
    ] {
        assert!(heavy.starts_with("/disk/noro"), "{}", heavy.display());
    }
    // What the bootstrapper reads, and what is small and per machine.
    for service in [
        dirs.config_file(),
        dirs.bootstrap_file(),
        dirs.optional_mods_file(),
        dirs.move_journal(),
        dirs.authlib_injector(),
        dirs.updates(),
    ] {
        assert!(service.starts_with("/svc"), "{}", service.display());
    }
    assert_eq!(dirs.root(), Path::new("/svc"));
}

#[test]
fn a_clone_follows_the_switch() {
    let dirs = LauncherDirectories::at(PathBuf::from("/svc"));
    let task = dirs.clone();
    dirs.set_data_root(Some(Path::new("/disk/noro")));
    assert_eq!(task.instances(), Path::new("/disk/noro/instances"));
    dirs.set_data_root(None);
    assert_eq!(task.instances(), Path::new("/svc/instances"));
}

#[test]
fn a_missing_or_foreign_data_root_is_unavailable() {
    let tmp = TempDir::new("dirs-available");
    let dirs = LauncherDirectories::at(tmp.path().join("svc"));

    dirs.set_data_root(Some(&tmp.path().join("unplugged")));
    assert!(!dirs.data_available());

    let foreign = tmp.path().join("foreign");
    std::fs::create_dir_all(&foreign).unwrap();
    dirs.set_data_root(Some(&foreign));
    assert!(!dirs.data_available(), "an empty folder is not ours");

    std::fs::write(foreign.join(DATA_MARKER), "").unwrap();
    assert!(dirs.data_available());
}

#[test]
fn ensure_leaves_the_data_root_alone() {
    let tmp = TempDir::new("dirs-ensure");
    let dirs = LauncherDirectories::at(tmp.path().join("svc"));
    dirs.set_data_root(Some(&tmp.path().join("unplugged")));
    dirs.ensure().unwrap();
    assert!(tmp.path().join("svc").join("updates").is_dir());
    assert!(!tmp.path().join("unplugged").exists());
}
