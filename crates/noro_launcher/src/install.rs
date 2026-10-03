//! Putting a verified core in place.
//!
//! Each file is written beside its final name and renamed over it. Written in
//! place, a download cut off halfway left a binary that failed its signature
//! check, and with the network gone there was nothing left to start.

use std::io::Write;
use std::path::{Path, PathBuf};

fn beside(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn write_synced(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let staged = beside(path, ".new");
    write_synced(&staged, bytes)?;
    std::fs::rename(&staged, path)
}

/// Where a running core is moved to make room for its replacement.
pub fn moved_aside(core: &Path) -> PathBuf {
    core.with_extension("old")
}

/// The binary, its signature and the version file, in that order. A crash
/// between the binary and the signature leaves a pair that fails the check,
/// and the next start downloads again instead of running something unchecked.
pub fn install(
    app_dir: &Path,
    core: &Path,
    bytes: &[u8],
    signature: &str,
    version: &str,
) -> std::io::Result<()> {
    let staged = beside(core, ".new");
    write_synced(&staged, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))?;
    }

    let sig = crate::verify::sig_path(core);
    let staged_sig = beside(&sig, ".new");
    write_synced(&staged_sig, signature.trim().as_bytes())?;

    // Windows won't replace a running exe, but it will rename one. That
    // happens when core is already open and the bootstrapper is started again.
    #[cfg(windows)]
    if core.exists() {
        let old = moved_aside(core);
        let _ = std::fs::remove_file(&old);
        std::fs::rename(core, &old)?;
    }

    std::fs::rename(&staged, core)?;
    std::fs::rename(&staged_sig, &sig)?;
    // Without this the launcher downloads itself again on every start.
    write_atomic(&app_dir.join("version"), version.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_all_three_files() {
        let dir = std::env::temp_dir().join(format!("noro-install-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let core = dir.join("core");
        std::fs::write(&core, b"old").unwrap();

        install(&dir, &core, b"new", " c2ln \n", "1.2.3").unwrap();

        assert_eq!(std::fs::read(&core).unwrap(), b"new");
        assert_eq!(
            std::fs::read_to_string(crate::verify::sig_path(&core)).unwrap(),
            "c2ln"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("version")).unwrap(),
            "1.2.3"
        );
        assert!(!beside(&core, ".new").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
