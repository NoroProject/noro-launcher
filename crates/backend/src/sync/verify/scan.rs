//! What gets checked, and what counts as an extra file.

use super::finding;
use crate::directories::safe_join;
use crate::sync::file_sync::is_protected;
use schema::{
    ArtifactKind, BuildManifest, FileEntry, IntegrityFinding, IntegrityKind, UserProfile,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Assets, libraries and the JRE are tens of thousands of files and gigabytes
/// on disk, and swapping one gains a player nothing worth half a minute of
/// staring at an empty window. Mods and configs are the point.
fn is_checked(kind: ArtifactKind) -> bool {
    matches!(
        kind,
        ArtifactKind::Mod | ArtifactKind::Config | ArtifactKind::ClientJar
    )
}

/// Manifest files that should be on disk right now.
pub fn expected_files<'a>(
    manifest: &'a BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> Vec<&'a FileEntry> {
    let off = crate::sync::file_sync::excluded_optional_files(manifest, enabled_optional, user);
    manifest
        .verified_files
        .iter()
        .filter(|f| f.side.needed_on_client())
        .filter(|f| f.matches_platform())
        .filter(|f| is_checked(manifest.kind_of(&f.path)))
        .filter(|f| !off.contains(&f.path))
        // An edit under user-managed is the player's edit, not a discrepancy.
        .filter(|f| !is_protected(&f.path, &manifest.user_managed_paths))
        .filter(|f| !is_protected(&f.path, &manifest.unmanaged_paths))
        .collect()
}

/// Deletes anything in the managed directories that the manifest doesn't list.
pub async fn remove_extras(
    instance_dir: &Path,
    manifest: &BuildManifest,
    expected: &[&FileEntry],
) -> Vec<IntegrityFinding> {
    // Only where every file belongs to the build. Walking the whole instance
    // means gigabytes of `saves/` and tens of thousands of assets.
    const MANAGED_DIRS: [&str; 2] = ["mods", "config"];

    let known: HashSet<&str> = expected.iter().map(|f| f.path.as_str()).collect();
    // A disabled optional mod is not an extra file — it sits there waiting to
    // be switched back on.
    let from_manifest: HashSet<&str> = manifest
        .verified_files
        .iter()
        .map(|f| f.path.as_str())
        .collect();

    let mut protected: Vec<String> = manifest.unmanaged_paths.clone();
    protected.extend(manifest.user_managed_paths.iter().cloned());
    protected.push(".noro/".to_string());

    let mut out = Vec::new();
    for dir in MANAGED_DIRS {
        let Some(root) = safe_join(instance_dir, dir) else {
            continue;
        };
        let Ok(mut entries) = tokio::fs::read_dir(&root).await else {
            continue;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            if !entry
                .file_type()
                .await
                .map(|t| t.is_file())
                .unwrap_or(false)
            {
                continue;
            }
            let rel = format!("{dir}/{}", entry.file_name().to_string_lossy());
            if known.contains(rel.as_str())
                || from_manifest.contains(rel.as_str())
                || is_protected(&rel, &protected)
            {
                continue;
            }
            let repaired = tokio::fs::remove_file(entry.path()).await.is_ok();
            out.push(finding(IntegrityKind::ExtraFile, &rel, None, repaired));
        }
    }
    out
}

/// Limited mods the player enabled without the permission for them.
///
/// The master strips those files from the manifest, so the enabled flag being
/// set at all means the client edited the list.
pub fn forbidden_optionals(
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> Vec<IntegrityFinding> {
    let known: HashMap<&str, bool> = manifest
        .optional_mods
        .iter()
        .map(|m| (m.name.as_str(), m.limited))
        .collect();

    enabled_optional
        .iter()
        .filter(|name| match known.get(name.as_str()) {
            Some(limited) => *limited && !user.can_use_optional(&manifest.server_id, name, true),
            // Name not in the manifest: almost always a choice saved before the
            // mod was dropped from the build. Flagging it would be a false
            // positive, and the client gains nothing — the files aren't there.
            None => false,
        })
        .map(|name| finding(IntegrityKind::ForbiddenOptionalMod, name, None, false))
        .collect()
}
