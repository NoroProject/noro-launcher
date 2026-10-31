//! The source archive: packed from a working copy, unpacked into the agent's
//! build directory.
//!
//! The working copy as it is, uncommitted edits included — "build what I'm
//! looking at" is the point of a local build. Ignored files stay out, and with
//! them `.cargo/config.toml`, so noro-shared comes from the commit pinned in
//! Cargo.lock exactly as in CI rather than from a sibling checkout.

use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::path::Path;

/// The directory a build keeps between jobs. Wiping it would turn every build
/// into a cold one, several minutes longer.
const KEEP: &str = "target";

pub fn pack(repo: &Path) -> Result<(Vec<u8>, usize)> {
    let out = std::process::Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(repo)
        .output()
        .context("running git ls-files")?;
    if !out.status.success() {
        bail!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let mut count = 0;
    for rel in out.stdout.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let rel = String::from_utf8_lossy(rel);
        let path = repo.join(rel.as_ref());
        // Tracked but deleted in the working copy: the build shouldn't see it.
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(mode(&meta));
        zip.start_file(rel.replace('\\', "/"), options)?;
        zip.write_all(&std::fs::read(&path)?)?;
        count += 1;
    }
    Ok((zip.finish()?.into_inner(), count))
}

#[cfg(unix)]
fn mode(meta: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o777
}

#[cfg(not(unix))]
fn mode(_: &std::fs::Metadata) -> u32 {
    0o644
}

/// Replace everything in `dest` except the build cache with the archive.
pub fn unpack(archive: &Path, dest: &Path) -> Result<usize> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(dest)? {
        let entry = entry?;
        if entry.file_name() == KEEP {
            continue;
        }
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        }
        .with_context(|| format!("clearing {}", path.display()))?;
    }

    let mut zip = zip::ZipArchive::new(std::fs::File::open(archive)?)?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        // A name like `../../x` would write outside the build directory.
        let Some(rel) = file.enclosed_name() else {
            bail!("unsafe path in the archive: {}", file.name());
        };
        let path = dest.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut bytes = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut bytes)?;
        std::fs::write(&path, bytes)?;
        #[cfg(unix)]
        if let Some(m) = file.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(m))?;
        }
    }
    Ok(zip.len())
}

/// `[workspace.package] version` — the one version every crate inherits.
pub fn workspace_version(root: &Path) -> Result<String> {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let mut in_package = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_package = line == "[workspace.package]";
        } else if in_package {
            if let Some(v) = line.strip_prefix("version") {
                let v = v
                    .trim_start()
                    .trim_start_matches('=')
                    .trim()
                    .trim_matches('"');
                return Ok(v.to_string());
            }
        }
    }
    bail!("no [workspace.package] version in {}", root.display())
}
