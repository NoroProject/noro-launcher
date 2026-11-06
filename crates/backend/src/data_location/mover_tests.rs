// Over 150 lines: one test per way a move can go, each needing its own
// install on disk.
//! Moves on disk, between two temporary folders. Whether they count as one
//! volume is the request's `same_volume` flag: a rename, or a copy that is
//! checked and then switched to.

use super::preflight_tests::install;
use super::*;
use crate::directories::{LauncherDirectories, DATA_MARKER};
use crate::test_http::TempDir;
use bridge::{DataMovePhase, DataMoveProgress};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

struct Setup {
    _tmp: TempDir,
    svc: PathBuf,
    target: PathBuf,
}

fn setup(tag: &str) -> Setup {
    let tmp = TempDir::new(tag);
    let svc = install(&tmp);
    let target = tmp.path().join("disk").join("Noro");
    Setup {
        svc,
        target,
        _tmp: tmp,
    }
}

fn request(s: &Setup, same_volume: bool) -> MoveRequest {
    MoveRequest {
        service_root: s.svc.clone(),
        journal: s.svc.join("data-move.json"),
        from: s.svc.clone(),
        to: s.target.clone(),
        same_volume,
    }
}

/// Runs a move, recording where the switch pointed and every progress report.
fn go(
    req: &MoveRequest,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(&DataMoveProgress),
) -> (Result<Finished, MoveError>, Option<PathBuf>) {
    let mut switched = None;
    let mut sink = |p: DataMoveProgress| on_progress(&p);
    let mut meter = Meter::new(cancel, &mut sink).unthrottled();
    let result = run(req, &mut meter, &mut |to: &Path| {
        switched = Some(to.to_path_buf());
        Ok(())
    });
    (result, switched)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// What the bootstrapper finds by its fixed path, and the config, unchanged.
fn assert_service_untouched(svc: &Path) {
    assert_eq!(read(&svc.join("config.json")), "{\"locale\": \"en\"}");
    assert_eq!(read(&svc.join("noro-launcher-core")), "core binary");
    assert_eq!(read(&svc.join("version")), "2.0.21");
    assert!(read(&svc.join("bootstrap.json")).contains("example.com"));
}

fn assert_moved(s: &Setup) {
    let instance = s.target.join("instances").join("server-a");
    assert_eq!(read(&instance.join("mods").join("a.jar")), "jar bytes");
    assert_eq!(read(&instance.join("saves/world/level.dat")), "a world");
    assert!(s.target.join("store").is_dir());
    assert!(s.target.join(DATA_MARKER).is_file());
    assert!(!s.svc.join("instances").exists());
    assert!(!s.svc.join("store").exists());
    assert!(
        !s.svc.join("data-move.json").exists(),
        "the journal is gone"
    );
    assert_service_untouched(&s.svc);
}

#[test]
fn on_one_volume_the_entries_are_renamed() {
    let s = setup("move-rename");
    let (result, switched) = go(&request(&s, true), &AtomicBool::new(false), |_| {});
    assert_eq!(result, Ok(Finished { leftover: None }));
    assert_eq!(switched, Some(s.target.clone()));
    assert_moved(&s);
}

#[test]
fn across_volumes_the_copy_is_checked_before_the_switch() {
    let s = setup("move-copy");
    #[cfg(unix)]
    let java = unix_extras(&s.svc);
    let mut phases = Vec::new();
    let (result, switched) = go(&request(&s, false), &AtomicBool::new(false), |p| {
        if phases.last() != Some(&p.phase) {
            phases.push(p.phase);
        }
    });
    assert_eq!(result, Ok(Finished { leftover: None }));
    assert_eq!(switched, Some(s.target.clone()));
    assert_eq!(
        phases,
        [
            DataMovePhase::Copying,
            DataMovePhase::Verifying,
            DataMovePhase::Switching,
            DataMovePhase::Cleaning
        ]
    );
    assert_moved(&s);
    #[cfg(unix)]
    check_unix_extras(&s.target, &java);
}

#[test]
fn moving_back_into_the_service_root_leaves_its_own_files_alone() {
    let s = setup("move-back");
    go(&request(&s, false), &AtomicBool::new(false), |_| {})
        .0
        .unwrap();

    let back = MoveRequest {
        from: s.target.clone(),
        to: s.svc.clone(),
        ..request(&s, false)
    };
    let (result, switched) = go(&back, &AtomicBool::new(false), |_| {});
    assert_eq!(result, Ok(Finished { leftover: None }));
    assert_eq!(switched, Some(s.svc.clone()));
    assert_eq!(
        read(&s.svc.join("instances/server-a/mods/a.jar")),
        "jar bytes"
    );
    assert!(
        !s.svc.join(DATA_MARKER).exists(),
        "no marker in the service root"
    );
    assert!(!s.target.exists(), "the emptied folder goes");
    assert_service_untouched(&s.svc);
}

#[test]
fn a_cancel_removes_the_partial_copy_and_switches_nothing() {
    let s = setup("move-cancel");
    let cancel = AtomicBool::new(false);
    let (result, switched) = go(&request(&s, false), &cancel, |p| {
        if p.done_files > 0 {
            cancel.store(true, Ordering::Relaxed);
        }
    });
    assert_eq!(result, Err(MoveError::Cancelled));
    assert_eq!(switched, None);
    assert!(!s.target.join("instances").exists());
    assert!(!s.target.join(DATA_MARKER).exists());
    assert_eq!(
        read(&s.svc.join("instances/server-a/mods/a.jar")),
        "jar bytes"
    );
    assert!(!s.svc.join("data-move.json").exists());
}

#[test]
fn a_copy_that_does_not_match_is_not_switched_to() {
    let s = setup("move-mismatch");
    let jar = s.target.join("instances/server-a/mods/a.jar");
    let (result, switched) = go(&request(&s, false), &AtomicBool::new(false), |p| {
        // The disk "loses" a write between the copy and the check.
        if p.phase == DataMovePhase::Verifying && p.done_bytes == 0 {
            std::fs::write(&jar, b"jar bytez").unwrap();
        }
    });
    let MoveError::Failed(refusal) = result.unwrap_err() else {
        panic!("not a failure");
    };
    assert_eq!(refusal.key, "data-move-error-verify");
    assert_eq!(switched, None);
    assert!(!s.target.join("instances").exists());
    assert_eq!(
        read(&s.svc.join("instances/server-a/mods/a.jar")),
        "jar bytes"
    );
}

#[test]
fn a_config_that_will_not_save_undoes_the_move() {
    let s = setup("move-config");
    let cancel = AtomicBool::new(false);
    let mut sink = |_p: DataMoveProgress| {};
    let mut meter = Meter::new(&cancel, &mut sink);
    let result = run(&request(&s, true), &mut meter, &mut |_| {
        Err(std::io::Error::other("disk full"))
    });
    assert_eq!(result, Err(Refusal::new("data-move-error-config").into()));
    assert_eq!(
        read(&s.svc.join("instances/server-a/mods/a.jar")),
        "jar bytes"
    );
    assert!(!s.target.join("instances").exists());
}

/// A crash in the middle of the copy: the progress callback panics, so
/// nothing after it — not even the undo — runs.
fn crash_mid_copy(s: &Setup) {
    let crashed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        go(&request(s, false), &AtomicBool::new(false), |p| {
            if p.done_files > 0 {
                panic!("power cut");
            }
        })
    }));
    assert!(crashed.is_err());
    assert!(
        s.target.join("instances").exists(),
        "a partial copy is left"
    );
}

#[test]
fn an_interrupted_copy_is_found_at_start_and_can_be_finished() {
    let s = setup("move-resume");
    crash_mid_copy(&s);

    let dirs = LauncherDirectories::at(s.svc.clone());
    let Startup::Interrupted(journal) = startup(&dirs) else {
        panic!("the journal should be found");
    };
    assert_eq!(
        (journal.from.clone(), journal.phase),
        (s.svc.clone(), Phase::Copying)
    );

    let resumed = MoveRequest {
        from: journal.from,
        to: journal.to,
        ..request(&s, false)
    };
    let (result, switched) = go(&resumed, &AtomicBool::new(false), |_| {});
    assert_eq!(result, Ok(Finished { leftover: None }));
    assert_eq!(switched, Some(s.target.clone()));
    assert_moved(&s);
}

#[test]
fn an_interrupted_copy_can_be_rolled_back() {
    let s = setup("move-rollback");
    crash_mid_copy(&s);
    let dirs = LauncherDirectories::at(s.svc.clone());
    let Startup::Interrupted(journal) = startup(&dirs) else {
        panic!("the journal should be found");
    };

    assert!(rollback(
        &journal,
        &s.svc,
        &dirs.move_journal(),
        &dirs.data_root()
    ));
    assert!(!s.target.join("instances").exists());
    assert!(!s.target.join(DATA_MARKER).exists());
    assert_eq!(
        read(&s.svc.join("instances/server-a/saves/world/level.dat")),
        "a world"
    );
    assert_eq!(startup(&dirs), Startup::Ready);
}

#[test]
fn a_rollback_never_deletes_the_folder_in_use() {
    let s = setup("move-rollback-guard");
    let journal = Journal {
        from: s.target.clone(),
        to: s.svc.clone(),
        phase: Phase::Copying,
    };
    assert!(!rollback(
        &journal,
        &s.svc,
        &s.svc.join("data-move.json"),
        &s.svc
    ));
    assert!(s.svc.join("instances/server-a/mods/a.jar").is_file());
}

#[test]
fn a_crash_after_the_switch_only_finishes_the_clean_up() {
    let s = setup("move-cleanup");
    let dirs = LauncherDirectories::at(s.svc.clone());
    std::fs::create_dir_all(&s.target).unwrap();
    std::fs::rename(s.svc.join("instances"), s.target.join("instances")).unwrap();
    std::fs::create_dir_all(s.svc.join("store/ab")).unwrap();
    // The journal still says copying; the config already says the new place.
    Journal {
        from: s.svc.clone(),
        to: s.target.clone(),
        phase: Phase::Copying,
    }
    .save(&dirs.move_journal())
    .unwrap();
    dirs.set_data_root(Some(&s.target));

    let Startup::Cleanup(journal) = startup(&dirs) else {
        panic!("should be read as clean-up");
    };
    assert_eq!(
        finish_cleanup(&journal, &s.svc, &dirs.move_journal(), &dirs.data_root()),
        None
    );
    assert!(!s.svc.join("store").exists());
    assert!(s.target.join("instances/server-a/mods/a.jar").is_file());
    assert!(!dirs.move_journal().exists());
    assert_service_untouched(&s.svc);
}

#[test]
fn a_disconnected_disk_is_reported_and_not_recreated() {
    let s = setup("move-unplugged");
    let dirs = LauncherDirectories::at(s.svc.clone());
    dirs.set_data_root(Some(&s.target));
    assert_eq!(startup(&dirs), Startup::Unavailable);
    dirs.ensure().unwrap();
    assert!(!s.target.exists());
}

/// An executable Java, a link, and a store file hard-linked into the instance.
#[cfg(unix)]
fn unix_extras(svc: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let java = svc.join("instances/server-a/runtime/bin/java");
    std::fs::create_dir_all(java.parent().unwrap()).unwrap();
    std::fs::write(&java, b"#!/bin/sh").unwrap();
    std::fs::set_permissions(&java, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::os::unix::fs::symlink("bin/java", svc.join("instances/server-a/runtime/java-link"))
        .unwrap();
    std::fs::create_dir_all(svc.join("store/ab")).unwrap();
    std::fs::write(svc.join("store/ab/abcdef"), b"shared library").unwrap();
    std::fs::hard_link(
        svc.join("store/ab/abcdef"),
        svc.join("instances/server-a/mods/lib.jar"),
    )
    .unwrap();
    PathBuf::from("instances/server-a/runtime/bin/java")
}

#[cfg(unix)]
fn check_unix_extras(target: &Path, java: &Path) {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let mode = std::fs::metadata(target.join(java))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o755, "Java stays executable");
    let link = std::fs::read_link(target.join("instances/server-a/runtime/java-link")).unwrap();
    assert_eq!(link, Path::new("bin/java"));
    let store = std::fs::metadata(target.join("store/ab/abcdef")).unwrap();
    let lib = std::fs::metadata(target.join("instances/server-a/mods/lib.jar")).unwrap();
    assert_eq!(store.ino(), lib.ino(), "a hard link stays one file");
}
