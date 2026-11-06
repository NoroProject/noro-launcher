// Over 150 lines: copying and checking the copy share the progress meter and
// the per-file helpers; apart, each would need its own copy of them.
//! Copying the data entries to another volume, and checking the copy.

use super::tree::{Entry, Kind, Tree};
use super::{io_failure, retry, MoveError, Refusal};
use bridge::{DataMovePhase, DataMoveProgress};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const CHUNK: usize = 1 << 20;
/// Often enough for a bar that moves, rarely enough not to flood the window.
const REPORT_EVERY: Duration = Duration::from_millis(200);

pub type Digests = HashMap<PathBuf, [u8; 20]>;

/// Progress and the cancel flag, threaded through every step of a move.
pub struct Meter<'a> {
    cancel: &'a AtomicBool,
    sink: &'a mut dyn FnMut(DataMoveProgress),
    now: DataMoveProgress,
    last: Option<Instant>,
    every: Duration,
}

impl<'a> Meter<'a> {
    pub fn new(cancel: &'a AtomicBool, sink: &'a mut dyn FnMut(DataMoveProgress)) -> Self {
        Self {
            cancel,
            sink,
            now: DataMoveProgress {
                phase: DataMovePhase::Copying,
                done_bytes: 0,
                total_bytes: 0,
                done_files: 0,
                total_files: 0,
            },
            last: None,
            every: REPORT_EVERY,
        }
    }

    /// Every step reported, for tests that act on a particular one.
    #[cfg(test)]
    pub fn unthrottled(mut self) -> Self {
        self.every = Duration::ZERO;
        self
    }

    pub fn phase(&mut self, phase: DataMovePhase, total_bytes: u64, total_files: u64) {
        self.now = DataMoveProgress {
            phase,
            done_bytes: 0,
            total_bytes,
            done_files: 0,
            total_files,
        };
        self.emit(true);
    }

    fn bytes(&mut self, n: u64) {
        self.now.done_bytes += n;
        self.emit(false);
    }

    fn file(&mut self) {
        self.now.done_files += 1;
        self.emit(false);
    }

    pub fn check(&self) -> Result<(), MoveError> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(MoveError::Cancelled)
        } else {
            Ok(())
        }
    }

    fn emit(&mut self, force: bool) {
        if force || self.last.is_none_or(|at| at.elapsed() >= self.every) {
            (self.sink)(self.now.clone());
            self.last = Some(Instant::now());
        }
    }
}

/// Copies every entry of `tree` from `from` to `to`. A file already at the
/// target with the source's size and modification time was finished by an
/// earlier, interrupted attempt and is not copied again.
pub fn copy_tree(
    from: &Path,
    to: &Path,
    tree: &Tree,
    meter: &mut Meter,
) -> Result<Digests, MoveError> {
    meter.phase(DataMovePhase::Copying, tree.bytes, tree.files);
    let mut digests = Digests::new();
    let mut first_name: HashMap<(u64, u64), PathBuf> = HashMap::new();
    let mut buf = vec![0u8; CHUNK];
    for entry in &tree.entries {
        meter.check()?;
        let (src, dst) = (from.join(&entry.rel), to.join(&entry.rel));
        match entry.kind {
            Kind::Dir => std::fs::create_dir_all(&dst).map_err(|e| io_failure(&dst, &e))?,
            Kind::Symlink => copy_link(&src, &dst)?,
            Kind::File => {
                if let Some(first) = entry.link.and_then(|key| first_name.get(&key)) {
                    link_again(first, &dst)?;
                } else {
                    if let Some(digest) = copy_file(&src, &dst, entry, &mut buf, meter)? {
                        digests.insert(entry.rel.clone(), digest);
                    }
                    if let Some(key) = entry.link {
                        first_name.insert(key, dst);
                    }
                }
            }
        }
        if entry.kind != Kind::Dir {
            meter.file();
        }
    }
    Ok(digests)
}

fn copy_file(
    src: &Path,
    dst: &Path,
    entry: &Entry,
    buf: &mut [u8],
    meter: &mut Meter,
) -> Result<Option<[u8; 20]>, MoveError> {
    if let Ok(done) = std::fs::symlink_metadata(dst) {
        if done.is_file()
            && done.len() == entry.len
            && entry.modified.is_some()
            && done.modified().ok() == entry.modified
        {
            meter.bytes(entry.len);
            return Ok(None);
        }
        remove_file(dst).map_err(|e| io_failure(dst, &e))?;
    }
    let mut input = retry(|| File::open(src)).map_err(|e| io_failure(src, &e))?;
    let mut output = retry(|| File::create(dst)).map_err(|e| io_failure(dst, &e))?;
    let mut hasher = Sha1::new();
    loop {
        meter.check()?;
        let n = retry(|| input.read(buf)).map_err(|e| io_failure(src, &e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        output
            .write_all(&buf[..n])
            .map_err(|e| io_failure(dst, &e))?;
        meter.bytes(n as u64);
    }
    // The time is what lets a resumed copy skip this file, so it is set while
    // the file is still open for writing; a read-only flag set first would
    // refuse it.
    if let Some(modified) = entry.modified {
        let _ = output.set_modified(modified);
    }
    drop(output);
    // Exec bits matter (the Java runtime); a filesystem without them, like
    // exFAT, refuses, and that is no reason to stop.
    let _ = std::fs::set_permissions(dst, entry.permissions.clone());
    Ok(Some(hasher.finalize().into()))
}

/// The second name of a hard-linked file. FAT and exFAT have no hard links;
/// there it becomes a file of its own.
fn link_again(first: &Path, dst: &Path) -> Result<(), MoveError> {
    if std::fs::symlink_metadata(dst).is_ok() {
        remove_file(dst).map_err(|e| io_failure(dst, &e))?;
    }
    std::fs::hard_link(first, dst)
        .or_else(|_| std::fs::copy(first, dst).map(|_| ()))
        .map_err(|e| io_failure(dst, &e).into())
}

/// Recreated as a link pointing where the original points. Following it
/// would copy whatever is behind it, possibly half a disk.
fn copy_link(src: &Path, dst: &Path) -> Result<(), MoveError> {
    let target = std::fs::read_link(src).map_err(|e| io_failure(src, &e))?;
    if std::fs::symlink_metadata(dst).is_ok() {
        remove_file(dst).map_err(|e| io_failure(dst, &e))?;
    }
    make_link(src, &target, dst).map_err(|_| Refusal::new("data-move-error-symlink").at(src).into())
}

#[cfg(unix)]
fn make_link(_src: &Path, target: &Path, dst: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, dst)
}

/// Needs Developer Mode or an elevated launcher; without either the player is
/// told which link stopped the move.
#[cfg(windows)]
fn make_link(src: &Path, target: &Path, dst: &Path) -> io::Result<()> {
    if std::fs::metadata(src).is_ok_and(|m| m.is_dir()) {
        std::os::windows::fs::symlink_dir(target, dst)
    } else {
        std::os::windows::fs::symlink_file(target, dst)
    }
}

#[cfg(not(any(unix, windows)))]
fn make_link(_src: &Path, _target: &Path, _dst: &Path) -> io::Result<()> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

/// Reads the copy back and compares it with the source: links by where they
/// point, files by size and SHA-1. The source's hash comes from the copy
/// pass; a file that pass skipped is read again here.
pub fn verify(
    from: &Path,
    to: &Path,
    tree: &Tree,
    digests: &Digests,
    meter: &mut Meter,
) -> Result<(), MoveError> {
    meter.phase(DataMovePhase::Verifying, tree.bytes, tree.files);
    let mut seen_links = std::collections::HashSet::new();
    let mut buf = vec![0u8; CHUNK];
    for entry in &tree.entries {
        meter.check()?;
        let (src, dst) = (from.join(&entry.rel), to.join(&entry.rel));
        let mismatch = || Refusal::new("data-move-error-verify").at(&dst);
        match entry.kind {
            Kind::Dir if !dst.is_dir() => return Err(mismatch().into()),
            Kind::Dir => continue,
            Kind::Symlink => {
                if std::fs::read_link(&dst).ok() != std::fs::read_link(&src).ok() {
                    return Err(mismatch().into());
                }
            }
            Kind::File => {
                let len = std::fs::metadata(&dst).map_err(|_| mismatch())?.len();
                if len != entry.len {
                    return Err(mismatch().into());
                }
                // Another name of a file already checked.
                if entry.link.is_none_or(|key| seen_links.insert(key)) {
                    let copied = hash_file(&dst, &mut buf, meter)?;
                    let original = match digests.get(&entry.rel) {
                        Some(digest) => *digest,
                        None => {
                            hash_file(&src, &mut buf, &mut Meter::new(meter.cancel, &mut |_| {}))?
                        }
                    };
                    if copied != original {
                        return Err(mismatch().into());
                    }
                }
            }
        }
        meter.file();
    }
    Ok(())
}

fn hash_file(path: &Path, buf: &mut [u8], meter: &mut Meter) -> Result<[u8; 20], MoveError> {
    let mut file = retry(|| File::open(path)).map_err(|e| io_failure(path, &e))?;
    let mut hasher = Sha1::new();
    loop {
        meter.check()?;
        let n = retry(|| file.read(buf)).map_err(|e| io_failure(path, &e))?;
        if n == 0 {
            return Ok(hasher.finalize().into());
        }
        hasher.update(&buf[..n]);
        meter.bytes(n as u64);
    }
}

/// A file left read-only by an earlier attempt can't be replaced on Windows
/// until the flag is cleared.
pub(super) fn remove_file(path: &Path) -> io::Result<()> {
    match retry(|| std::fs::remove_file(path)) {
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            make_writable(path);
            retry(|| std::fs::remove_file(path))
        }
        done => done,
    }
}

#[cfg(unix)]
pub(super) fn make_writable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        let mode = meta.permissions().mode() | 0o200;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
    }
}

/// On Windows this clears the read-only attribute and nothing else; the lint
/// is about Unix, where it would make a file writable for everyone.
#[cfg(not(unix))]
#[allow(clippy::permissions_set_readonly_false)]
pub(super) fn make_writable(path: &Path) {
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        let mut perms = meta.permissions();
        perms.set_readonly(false);
        let _ = std::fs::set_permissions(path, perms);
    }
}
