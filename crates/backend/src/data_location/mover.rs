// Over 150 lines: the move with its undo and its clean-up — each knows which
// half of the other's work it may find on disk.
//! Carrying out a move, undoing one, and finishing one; see the module docs.

use super::copy::{copy_tree, make_writable, verify};
use super::journal::{Journal, Phase};
use super::{io_failure, retry, scan, MoveError, Refusal};
use crate::directories::{DATA_ENTRIES, DATA_MARKER};
use bridge::DataMovePhase;
use std::io;
use std::path::{Path, PathBuf};

pub use super::copy::Meter;

pub struct MoveRequest {
    pub service_root: PathBuf,
    pub journal: PathBuf,
    pub from: PathBuf,
    pub to: PathBuf,
    pub same_volume: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Finished {
    /// What could not be deleted from the old place, for the player to remove.
    pub leftover: Option<PathBuf>,
}

/// Moves the data entries from `req.from` to `req.to`, then calls `switch`
/// with the new root — the one step after which the move counts as done.
/// Any failure before it puts things back the way they were.
pub fn run(
    req: &MoveRequest,
    meter: &mut Meter,
    switch: &mut dyn FnMut(&Path) -> io::Result<()>,
) -> Result<Finished, MoveError> {
    let mut phase = if req.same_volume {
        Phase::Renaming
    } else {
        Phase::Copying
    };
    record(req, phase)?;
    let moved = begin(req).and_then(|()| match phase {
        Phase::Renaming => match rename_entries(&req.from, &req.to) {
            Err(Renamed::CrossDevice) => {
                // A mount point or a link inside the data root: the probe
                // saw one volume, an entry lives on another.
                phase = Phase::Copying;
                record(req, phase)?;
                copy_and_verify(req, meter)
            }
            other => other.map_err(Renamed::into_error),
        },
        _ => copy_and_verify(req, meter),
    });
    if let Err(e) = moved {
        undo(req, phase);
        return Err(e);
    }

    meter.phase(DataMovePhase::Switching, 0, 0);
    if phase == Phase::Copying {
        flush_to_disk();
    }
    if let Err(e) = switch(&req.to) {
        tracing::error!(error = %e, "the new data root was not saved");
        undo(req, phase);
        return Err(Refusal::new("data-move-error-config").into());
    }
    if let Err(e) = record(req, Phase::Cleanup) {
        // The config already says `to`, which is what a journal left behind
        // is read against.
        tracing::warn!(?e, "move journal not updated for clean-up");
    }
    let leftover = clean_up(req, meter);
    Journal::clear(&req.journal);
    Ok(Finished { leftover })
}

/// Takes back an interrupted move that never reached the switch. Refuses
/// when `current` — the data root in use — is the journal's target: that
/// copy is no longer partial, it is the one the game runs from.
pub fn rollback(
    journal: &Journal,
    service_root: &Path,
    journal_path: &Path,
    current: &Path,
) -> bool {
    if journal.to == current {
        return false;
    }
    let req = MoveRequest {
        service_root: service_root.to_path_buf(),
        journal: journal_path.to_path_buf(),
        from: journal.from.clone(),
        to: journal.to.clone(),
        same_volume: journal.phase == Phase::Renaming,
    };
    undo(&req, journal.phase)
}

/// Deletes the old copy of a move whose switch went through. Deletes nothing
/// if `current`, the data root in use, is that old copy after all.
pub fn finish_cleanup(
    journal: &Journal,
    service_root: &Path,
    journal_path: &Path,
    current: &Path,
) -> Option<PathBuf> {
    if journal.from == current {
        Journal::clear(journal_path);
        return None;
    }
    let req = MoveRequest {
        service_root: service_root.to_path_buf(),
        journal: journal_path.to_path_buf(),
        from: journal.from.clone(),
        to: journal.to.clone(),
        same_volume: false,
    };
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let leftover = clean_up(&req, &mut Meter::new(&cancel, &mut |_| {}));
    Journal::clear(journal_path);
    leftover
}

fn record(req: &MoveRequest, phase: Phase) -> Result<(), MoveError> {
    let journal = Journal {
        from: req.from.clone(),
        to: req.to.clone(),
        phase,
    };
    journal
        .save(&req.journal)
        .map_err(|e| io_failure(&req.journal, &e).into())
}

fn begin(req: &MoveRequest) -> Result<(), MoveError> {
    std::fs::create_dir_all(&req.to).map_err(|e| io_failure(&req.to, &e))?;
    if req.to != req.service_root {
        std::fs::write(req.to.join(DATA_MARKER), b"noro launcher data\n")
            .map_err(|e| io_failure(&req.to, &e))?;
    }
    Ok(())
}

fn copy_and_verify(req: &MoveRequest, meter: &mut Meter) -> Result<(), MoveError> {
    let tree = scan(&req.from)?;
    let digests = copy_tree(&req.from, &req.to, &tree, meter)?;
    verify(&req.from, &req.to, &tree, &digests, meter)
}

enum Renamed {
    CrossDevice,
    Failed(Refusal),
}

impl Renamed {
    fn into_error(self) -> MoveError {
        match self {
            Renamed::Failed(r) => r.into(),
            Renamed::CrossDevice => Refusal::new("data-move-error-io").into(),
        }
    }
}

/// Renames each entry still in `from` into `to`. An entry already in `to` and
/// gone from `from` was moved by an earlier attempt.
fn rename_entries(from: &Path, to: &Path) -> Result<(), Renamed> {
    let mut done = Vec::new();
    for name in DATA_ENTRIES {
        let (src, dst) = (from.join(name), to.join(name));
        if std::fs::symlink_metadata(&src).is_err() {
            continue;
        }
        if std::fs::symlink_metadata(&dst).is_ok() {
            rename_back(&done, from, to);
            return Err(Renamed::Failed(
                Refusal::new("data-move-refused-not-empty").at(&dst),
            ));
        }
        if let Err(e) = retry(|| std::fs::rename(&src, &dst)) {
            rename_back(&done, from, to);
            return Err(if crosses_devices(&e) {
                Renamed::CrossDevice
            } else {
                Renamed::Failed(io_failure(&src, &e))
            });
        }
        done.push(name);
    }
    Ok(())
}

fn rename_back(names: &[&str], from: &Path, to: &Path) {
    for name in names {
        if let Err(e) = std::fs::rename(to.join(name), from.join(name)) {
            tracing::error!(name, error = %e, "could not rename back");
        }
    }
}

fn crosses_devices(e: &io::Error) -> bool {
    // ERROR_NOT_SAME_DEVICE, for a std that does not map it.
    e.kind() == io::ErrorKind::CrossesDevices || (cfg!(windows) && e.raw_os_error() == Some(17))
}

/// Puts things back as they were before the move: entries renamed into `to`
/// go back, a partial copy is deleted. The journal goes only once that worked,
/// so a copy that would not delete is offered again on the next start.
fn undo(req: &MoveRequest, phase: Phase) -> bool {
    let mut clean = true;
    for name in DATA_ENTRIES {
        let (src, dst) = (req.from.join(name), req.to.join(name));
        if std::fs::symlink_metadata(&dst).is_err() {
            continue;
        }
        let result = match phase {
            Phase::Renaming if std::fs::symlink_metadata(&src).is_err() => {
                std::fs::rename(&dst, &src)
            }
            // Both present while renaming: the target one is someone else's.
            Phase::Renaming => Ok(()),
            _ => remove_tree(&dst),
        };
        if let Err(e) = result {
            tracing::error!(path = %dst.display(), error = %e, "could not undo the move");
            clean = false;
        }
    }
    if clean {
        if req.to != req.service_root {
            let _ = std::fs::remove_file(req.to.join(DATA_MARKER));
        }
        Journal::clear(&req.journal);
    }
    clean
}

/// Deletes the entries left in `from`, and `from` itself once it is empty and
/// not the service root. Never anything outside the data entries.
fn clean_up(req: &MoveRequest, meter: &mut Meter) -> Option<PathBuf> {
    meter.phase(DataMovePhase::Cleaning, 0, 0);
    let mut leftover = None;
    for name in DATA_ENTRIES {
        let old = req.from.join(name);
        if let Err(e) = remove_tree(&old) {
            tracing::warn!(path = %old.display(), error = %e, "old data not removed");
            leftover.get_or_insert(old);
        }
    }
    if req.from != req.service_root {
        let _ = std::fs::remove_file(req.from.join(DATA_MARKER));
        let _ = std::fs::remove_dir(&req.from);
    }
    leftover
}

/// `remove_dir_all`, with read-only flags cleared when they are in the way
/// and transient locks waited out.
fn remove_tree(path: &Path) -> io::Result<()> {
    let attempt = || match retry(|| std::fs::remove_dir_all(path)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        done => done,
    };
    match attempt() {
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            for item in walkdir::WalkDir::new(path).into_iter().flatten() {
                make_writable(item.path());
            }
            attempt()
        }
        done => done,
    }
}

/// The old copy is deleted next, so the new one has to be on the disk and not
/// only in the cache. One `sync` covers every file; an `fsync` per file costs
/// minutes on a USB stick with tens of thousands of small files. Windows
/// writes removable drives through by default and has no unprivileged
/// equivalent.
fn flush_to_disk() {
    #[cfg(unix)]
    // SAFETY: takes no arguments and cannot fail.
    unsafe {
        libc::sync();
    }
}
