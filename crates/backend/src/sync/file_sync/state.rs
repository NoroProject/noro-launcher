//! What is installed in an instance, and what runs it.

use crate::directories::safe_join;
use schema::{ArtifactKind, BuildManifest};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

/// Says which build is installed: its version on the first line, for people and
/// diagnostics, and a fingerprint of its file list on the second.
pub fn version_marker(instance_dir: &Path) -> PathBuf {
    instance_dir.join(".noro-build")
}

/// The version alone can't be trusted: a build re-imported in place keeps it
/// while its files change, and the button went on offering «Play».
pub fn marker_contents(manifest: &BuildManifest) -> String {
    format!("{}\n{}", manifest.version, fingerprint(manifest))
}

pub fn build_state(instance_dir: &Path, manifest: &BuildManifest) -> bridge::BuildState {
    let Ok(marker) = std::fs::read_to_string(version_marker(instance_dir)) else {
        return bridge::BuildState::Missing;
    };
    let mut lines = marker.lines().map(str::trim);
    let version = lines.next().unwrap_or_default();
    let current = match lines.next() {
        Some(installed) => installed == fingerprint(manifest),
        // Written before the fingerprint: the version is all there is to go on.
        None => version == manifest.version,
    };
    if current {
        bridge::BuildState::Ready
    } else {
        bridge::BuildState::Outdated
    }
}

/// What the disk alone tells before the manifest arrives: whether there is a
/// build at all. Whether it is current takes the manifest.
pub fn installed_state(instance_dir: &Path) -> bridge::BuildState {
    if version_marker(instance_dir).exists() {
        bridge::BuildState::Ready
    } else {
        bridge::BuildState::Missing
    }
}

/// Paths and hashes of every file, in a fixed order: the same list gives the
/// same fingerprint whatever order the master sent it in.
fn fingerprint(manifest: &BuildManifest) -> String {
    let mut files: Vec<(&str, &str)> = manifest
        .verified_files
        .iter()
        .map(|f| (f.path.as_str(), f.sha1.as_str()))
        .collect();
    files.sort_unstable();
    let mut hasher = Sha1::new();
    for (path, sha1) in files {
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(sha1.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}

pub fn find_java(instance_dir: &Path, manifest: &BuildManifest) -> Option<PathBuf> {
    for f in &manifest.verified_files {
        // A manifest carries runtimes for several platforms; take ours.
        if !f.matches_platform() {
            continue;
        }
        if manifest.kind_of(&f.path) == ArtifactKind::Java
            && (f.path.ends_with("/bin/java") || f.path.ends_with("/bin/java.exe"))
        {
            return safe_join(instance_dir, &f.path);
        }
    }
    None
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
