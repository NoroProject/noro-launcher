//! What is installed in an instance, and what runs it.

use crate::directories::safe_join;
use schema::{ArtifactKind, BuildManifest};
use std::path::{Path, PathBuf};

/// Holds the version of the installed build.
pub fn version_marker(instance_dir: &Path) -> PathBuf {
    instance_dir.join(".noro-build")
}

pub fn build_state(instance_dir: &Path, manifest: &BuildManifest) -> bridge::BuildState {
    match std::fs::read_to_string(version_marker(instance_dir)) {
        Ok(installed) if installed.trim() == manifest.version => bridge::BuildState::Ready,
        Ok(_) => bridge::BuildState::Outdated,
        Err(_) => bridge::BuildState::Missing,
    }
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
