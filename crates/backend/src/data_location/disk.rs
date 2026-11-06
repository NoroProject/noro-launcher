//! What a move needs to know about the disks: how much room there is, and
//! whether two folders share a volume. Behind a trait so tests can play a full
//! disk or a second drive with two temporary folders.

use std::io;
use std::path::Path;

pub trait Disk: Send + Sync {
    /// Bytes this user can still write where `path` lives.
    fn available_space(&self, path: &Path) -> io::Result<u64>;
    /// Whether a rename from `from` into `to` would work. Both exist.
    fn same_volume(&self, from: &Path, to: &Path) -> bool;
}

pub struct RealDisk;

impl Disk for RealDisk {
    fn available_space(&self, path: &Path) -> io::Result<u64> {
        crate::disk_space::available(path)
    }

    /// Asked by doing it: there is no portable way to name the volume a path
    /// is on, and a rename is exactly what the move will try. Mount points,
    /// bind mounts and subst'ed drive letters all answer correctly this way.
    fn same_volume(&self, from: &Path, to: &Path) -> bool {
        let name = format!(".noro-volume-probe-{}", std::process::id());
        let probe = from.join(&name);
        if std::fs::write(&probe, b"").is_err() {
            return false;
        }
        let moved = to.join(&name);
        let same = std::fs::rename(&probe, &moved).is_ok();
        let _ = std::fs::remove_file(if same { &moved } else { &probe });
        same
    }
}
