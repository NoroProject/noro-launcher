// Over 150 lines: one test per check, each with its own folders on disk.
//! The checks a folder goes through before a move. Two temporary folders play
//! the disks; [`FakeDisk`] says how much room there is and whether they share
//! a volume.

use super::*;
use crate::test_http::TempDir;
use std::path::{Path, PathBuf};

pub(super) struct FakeDisk {
    pub free: u64,
    pub same: bool,
}

impl Disk for FakeDisk {
    fn available_space(&self, _path: &Path) -> std::io::Result<u64> {
        Ok(self.free)
    }
    fn same_volume(&self, _from: &Path, _to: &Path) -> bool {
        self.same
    }
}

const ROOMY: FakeDisk = FakeDisk {
    free: u64::MAX,
    same: false,
};

/// A service root with what the bootstrapper reads and a small instance.
pub(super) fn install(tmp: &TempDir) -> PathBuf {
    let root = tmp.path().join("svc");
    let files: [(&str, &[u8]); 6] = [
        ("config.json", b"{\"locale\": \"en\"}"),
        (
            "bootstrap.json",
            b"{\"master_url\": \"https://example.com\"}",
        ),
        ("noro-launcher-core", b"core binary"),
        ("version", b"2.0.21"),
        ("instances/server-a/mods/a.jar", b"jar bytes"),
        ("instances/server-a/saves/world/level.dat", b"a world"),
    ];
    for (rel, bytes) in files {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    std::fs::create_dir_all(root.join("store")).unwrap();
    root
}

#[test]
fn the_current_folder_and_anything_inside_it_are_refused() {
    let tmp = TempDir::new("preflight-inside");
    let svc = install(&tmp);
    let same = plan(&svc, &svc, &svc, &ROOMY).unwrap_err();
    assert_eq!(same.key, "data-move-refused-same");
    let inside = plan(&svc, &svc, &svc.join("instances").join("new"), &ROOMY).unwrap_err();
    assert_eq!(inside.key, "data-move-refused-inside");
}

#[test]
fn the_service_root_is_off_limits_once_the_data_lives_elsewhere() {
    let tmp = TempDir::new("preflight-service");
    let svc = install(&tmp);
    let data = tmp.path().join("disk");
    std::fs::create_dir_all(&data).unwrap();
    let refused = plan(&svc, &data, &svc.join("logs"), &ROOMY).unwrap_err();
    assert_eq!(refused.key, "data-move-refused-service");
}

#[test]
fn a_busy_folder_gets_a_folder_of_our_own_inside_it() {
    let tmp = TempDir::new("preflight-busy");
    let svc = install(&tmp);
    let games = tmp.path().join("Games");
    std::fs::create_dir_all(games.join("Other Game")).unwrap();

    let planned = plan(&svc, &svc, &games, &ROOMY).unwrap();
    assert_eq!(planned.target, games.join(schema::launcher_dir_name()));

    // Our own folder in there is busy too: that is a refusal, not a deeper one.
    std::fs::create_dir_all(games.join(schema::launcher_dir_name()).join("x")).unwrap();
    let refused = plan(&svc, &svc, &games, &ROOMY).unwrap_err();
    assert_eq!(refused.key, "data-move-refused-not-empty");
}

#[test]
fn system_litter_and_our_marker_count_as_empty() {
    let tmp = TempDir::new("preflight-litter");
    let svc = install(&tmp);
    let target = tmp.path().join("Noro");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join(".DS_Store"), b"").unwrap();
    std::fs::write(target.join(crate::directories::DATA_MARKER), b"").unwrap();

    let planned = plan(&svc, &svc, &target, &ROOMY).unwrap();
    assert_eq!(planned.target, target);
    assert!(!planned.same_volume);
    assert!(
        planned.bytes >= 16,
        "both files of the instance are counted"
    );
}

#[test]
fn another_disk_needs_room_for_the_data_and_a_margin() {
    let tmp = TempDir::new("preflight-space");
    let svc = install(&tmp);
    let target = tmp.path().join("Noro");

    let full = FakeDisk {
        free: 1024,
        same: false,
    };
    let refused = plan(&svc, &svc, &target, &full).unwrap_err();
    assert_eq!(refused.key, "data-move-refused-space");
    assert_eq!(refused.args["free"], "1024");
    assert!(refused.args["need"].parse::<u64>().unwrap() > 1024);

    // A rename takes no room at all.
    let same = FakeDisk {
        free: 0,
        same: true,
    };
    assert!(plan(&svc, &svc, &target, &same).unwrap().same_volume);
}

#[test]
fn moving_back_needs_only_the_data_entries_to_be_absent() {
    let tmp = TempDir::new("preflight-back");
    let svc = install(&tmp);
    let data = tmp.path().join("disk");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::rename(svc.join("instances"), data.join("instances")).unwrap();
    std::fs::remove_dir(svc.join("store")).unwrap();

    let planned = plan(&svc, &data, &svc, &ROOMY).unwrap();
    assert_eq!(planned.target, svc);

    std::fs::create_dir_all(svc.join("instances").join("leftover")).unwrap();
    let refused = plan(&svc, &data, &svc, &ROOMY).unwrap_err();
    assert_eq!(refused.key, "data-move-refused-not-empty");
}

#[cfg(unix)]
#[test]
fn a_folder_without_write_access_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new("preflight-readonly");
    let svc = install(&tmp);
    let locked = tmp.path().join("locked");
    std::fs::create_dir_all(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();

    let result = plan(&svc, &svc, &locked.join("Noro"), &ROOMY);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    // Root writes anywhere; the check can't be seen to fail then.
    if unsafe { libc::geteuid() } != 0 {
        assert_eq!(result.unwrap_err().key, "data-move-refused-readonly");
    }
}
