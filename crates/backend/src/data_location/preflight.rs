// Over 150 lines: every check a folder has to pass before a move, in the
// order the player would want to hear about them.
//! Checking a folder the player picked, before anything is written.

use super::{scan, Disk, Refusal};
use crate::directories::{DATA_ENTRIES, DATA_MARKER};
use crate::disk_space::nearest_existing;
use std::path::{Path, PathBuf};

pub use bridge::DataMovePlan as Plan;

/// Files the system drops into any folder a file manager has shown. A folder
/// holding only these is as good as empty.
const SYSTEM_LITTER: [&str; 3] = [".DS_Store", "Thumbs.db", "desktop.ini"];

/// Room left over on top of the data itself: the filesystem's own overhead,
/// and the game writing logs while it plays afterwards.
fn margin(bytes: u64) -> u64 {
    (bytes / 20).max(256 << 20)
}

/// Where the files would go if the player picked `chosen`, and whether they
/// can. `current` is the data root in use, `service_root` the launcher's own
/// folder.
pub fn plan(
    service_root: &Path,
    current: &Path,
    chosen: &Path,
    disk: &dyn Disk,
) -> Result<Plan, Refusal> {
    let chosen = std::path::absolute(chosen).map_err(|e| {
        Refusal::new("data-move-refused-create")
            .at(chosen)
            .arg("detail", e)
    })?;
    let target = resolve(service_root, &chosen);

    let (target_real, current_real, service_real) =
        (real(&target), real(current), real(service_root));
    if target_real == current_real {
        return Err(Refusal::new("data-move-refused-same"));
    }
    if target_real.starts_with(&current_real) {
        return Err(Refusal::new("data-move-refused-inside"));
    }
    if target_real != service_real && target_real.starts_with(&service_real) {
        return Err(Refusal::new("data-move-refused-service"));
    }
    check_empty(&target, target_real == service_real)?;

    let existing = nearest_existing(&target).map_err(|e| {
        Refusal::new("data-move-refused-create")
            .at(&target)
            .arg("detail", e)
    })?;
    let probe = Probe::create(&existing)
        .ok_or_else(|| Refusal::new("data-move-refused-readonly").at(&existing))?;

    let tree = scan(current)?;
    let same_volume = disk.same_volume(current, &probe.0);
    drop(probe);
    let free = disk.available_space(&existing).ok();
    if let Some(free) = free.filter(|_| !same_volume) {
        let need = tree.bytes + margin(tree.bytes);
        if free < need {
            return Err(Refusal::new("data-move-refused-space")
                .arg("need", need)
                .arg("free", free));
        }
    }
    Ok(Plan {
        target,
        bytes: tree.bytes,
        free,
        same_volume,
    })
}

/// The folder picked, or a new folder inside it when the one picked already
/// holds something — "D:\" or "D:\Games" — so a player who points at a
/// whole disk gets `D:\noro-launcher` rather than a refusal.
fn resolve(service_root: &Path, chosen: &Path) -> PathBuf {
    if real(chosen) == real(service_root) || is_empty_or_ours(chosen) {
        chosen.to_path_buf()
    } else {
        chosen.join(schema::launcher_dir_name())
    }
}

/// The service root is never empty; moving back into it only needs the data
/// entries to be absent there.
fn check_empty(target: &Path, is_service_root: bool) -> Result<(), Refusal> {
    if is_service_root {
        for name in DATA_ENTRIES {
            let entry = target.join(name);
            if std::fs::read_dir(&entry).is_ok_and(|mut d| d.next().is_some()) {
                return Err(Refusal::new("data-move-refused-not-empty").at(&entry));
            }
        }
        return Ok(());
    }
    if is_empty_or_ours(target) {
        Ok(())
    } else {
        Err(Refusal::new("data-move-refused-not-empty").at(target))
    }
}

/// Absent, empty, or holding only system litter and the marker a previous
/// move left behind.
fn is_empty_or_ours(dir: &Path) -> bool {
    match std::fs::read_dir(dir) {
        Ok(entries) => entries.flatten().all(|e| {
            let name = e.file_name();
            name == DATA_MARKER || SYSTEM_LITTER.iter().any(|l| name == *l)
        }),
        Err(e) => e.kind() == std::io::ErrorKind::NotFound,
    }
}

/// A folder with a file in it, made where the target folder would be made.
/// Trying is the only answer that holds on every platform: permission bits say
/// nothing about ACLs, read-only mounts or a locked SD card. A folder and not
/// just a file, because the root of a Windows system drive lets users create
/// folders but not files.
struct Probe(PathBuf);

impl Probe {
    fn create(parent: &Path) -> Option<Self> {
        let dir = parent.join(format!(".noro-move-probe-{}", std::process::id()));
        std::fs::create_dir(&dir).ok()?;
        let probe = Probe(dir);
        std::fs::write(probe.0.join("probe"), b"").ok()?;
        Some(probe)
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The path with links and `..` resolved as far as it exists, for comparing:
/// `/tmp` and `/private/tmp` are one folder on macOS, and a path not created
/// yet is compared through its nearest existing parent.
fn real(path: &Path) -> PathBuf {
    let Ok(existing) = nearest_existing(path) else {
        return path.to_path_buf();
    };
    let rest = path.strip_prefix(&existing).unwrap_or(Path::new(""));
    match existing.canonicalize() {
        Ok(base) if rest.as_os_str().is_empty() => base,
        Ok(base) => base.join(rest),
        Err(_) => path.to_path_buf(),
    }
}
