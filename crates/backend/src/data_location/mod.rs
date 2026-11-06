//! Moving the data root — instances and the store — to another folder or disk.
//!
//! The service root never moves (see [`crate::directories`]). The order below
//! is what keeps an interrupted move from breaking the install:
//!
//! 1. a journal in the service root records that a move is under way;
//! 2. on the same volume the entries are renamed, which is instant; across
//!    volumes they are copied, and the copy is checked file by file against
//!    the originals;
//! 3. only then does `config.json` get the new path, in one atomic write;
//! 4. the old copy is deleted. A file that won't go is left for the player
//!    and named, not treated as a failure: the move itself is done.
//!
//! Until step 3 the config points at the old folder, which nothing has
//! touched, so a crash anywhere before it costs only the partial copy — and
//! the journal finds that on the next start, to finish the move or remove it.
//!
//! The check reads every copied file back and compares its SHA-1 with the
//! source's, rather than a sample. The originals are deleted right after;
//! worlds and screenshots exist nowhere else; and the drives players move to —
//! USB sticks, cheap external disks — are the ones that lose writes. The
//! source is hashed while it is copied, so the price is one extra read of the
//! copy, not two.

mod copy;
mod disk;
mod guard;
mod journal;
mod mover;
mod preflight;
mod status;
mod tree;

pub use disk::{Disk, RealDisk};
pub use guard::{DataUse, MoveGuard, UseGuard};
pub use journal::{Journal, Phase};
pub use mover::{finish_cleanup, rollback, run, Finished, Meter, MoveRequest};
pub use preflight::{plan, Plan};
pub use status::{startup, Startup};
pub use tree::{scan, Tree};

use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::time::Duration;

/// Why a move can't start or did not finish: a translation key and its
/// arguments, for the window to put into words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub key: &'static str,
    pub args: BTreeMap<String, String>,
}

impl Refusal {
    pub fn new(key: &'static str) -> Self {
        Self {
            key,
            args: BTreeMap::new(),
        }
    }

    pub fn arg(mut self, name: &str, value: impl ToString) -> Self {
        self.args.insert(name.to_string(), value.to_string());
        self
    }

    pub fn at(self, path: &Path) -> Self {
        self.arg("path", path.display())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveError {
    Cancelled,
    Failed(Refusal),
}

impl From<Refusal> for MoveError {
    fn from(r: Refusal) -> Self {
        MoveError::Failed(r)
    }
}

/// Names the file and, as far as it can, the reason in the player's terms.
pub(crate) fn io_failure(path: &Path, e: &io::Error) -> Refusal {
    if is_locked(e) {
        return Refusal::new("data-move-error-locked").at(path);
    }
    match e.kind() {
        io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded => {
            Refusal::new("data-move-error-space")
        }
        io::ErrorKind::PermissionDenied => Refusal::new("data-move-error-access").at(path),
        _ => Refusal::new("data-move-error-io").at(path).arg("detail", e),
    }
}

/// Another process has the file open in a way that excludes us: on Windows
/// that is mostly an antivirus or the search indexer looking at a file we just
/// wrote, and it lets go within a second or two.
fn is_locked(e: &io::Error) -> bool {
    // ERROR_SHARING_VIOLATION and ERROR_LOCK_VIOLATION.
    e.kind() == io::ErrorKind::ResourceBusy
        || (cfg!(windows) && matches!(e.raw_os_error(), Some(32 | 33)))
}

/// An antivirus holding a file also shows up as plain access denied on Windows,
/// so that is worth a few more tries there too.
fn is_transient(e: &io::Error) -> bool {
    is_locked(e) || (cfg!(windows) && e.kind() == io::ErrorKind::PermissionDenied)
}

/// Runs `op` again on a transient failure, waiting a little longer each time —
/// about four seconds in all before the error is the player's to see.
pub(crate) fn retry<T>(mut op: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    let mut delay = Duration::from_millis(100);
    for _ in 0..4 {
        match op() {
            Err(e) if is_transient(&e) => {
                std::thread::sleep(delay);
                delay *= 3;
            }
            done => return done,
        }
    }
    op()
}

#[cfg(test)]
mod mover_tests;
#[cfg(test)]
mod preflight_tests;
