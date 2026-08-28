//! Checks the instance against the manifest immediately before launch.
//!
//! `clean_extra` runs during sync, and nothing looks at the directory again
//! between sync and launch. This repeats the check at the last moment: extras
//! are deleted, discrepancies go to the master as a flag.
//!
//! The player-facing reaction is deliberately quiet — "build files restored"
//! and the game starts. An honest player with a bad disk or an overeager
//! antivirus notices nothing, and someone who dropped a mod in on purpose
//! doesn't learn where the check is.

mod cache;
mod scan;

use crate::directories::safe_join;
use cache::HashCache;
use schema::{BuildManifest, IntegrityFinding, IntegrityKind, IntegrityReport, UserProfile};
use std::path::Path;

/// Checks the instance and repairs whatever a deletion can repair.
pub async fn verify_before_launch(
    instance_dir: &Path,
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> IntegrityReport {
    let mut cache = HashCache::load(instance_dir).await;
    let mut findings = Vec::new();
    let mut checked = 0u32;

    let expected = scan::expected_files(manifest, enabled_optional, user);
    for f in &expected {
        let Some(path) = safe_join(instance_dir, &f.path) else {
            continue;
        };
        match cache.sha1_of(&path).await {
            None => findings.push(finding(IntegrityKind::MissingFile, &f.path, None, false)),
            Some(actual) if actual != f.sha1 => {
                checked += 1;
                findings.push(finding(
                    IntegrityKind::ModifiedFile,
                    &f.path,
                    Some(format!("expected {}, on disk {}", f.sha1, actual)),
                    false,
                ));
            }
            Some(_) => checked += 1,
        }
    }

    findings.extend(scan::remove_extras(instance_dir, manifest, &expected).await);
    findings.extend(scan::forbidden_optionals(manifest, enabled_optional, user));

    // Blocked files override every path rule — they reach into directories
    // sync never touches.
    let blocked = crate::sync::blocklist::enforce(instance_dir, &manifest.blocked_files).await;
    let block_launch = blocked.block_launch;
    findings.extend(blocked.findings);

    // Inventory of the unsynced directories: what the player put there.
    let known: Vec<String> = manifest
        .verified_files
        .iter()
        .map(|f| f.path.clone())
        .collect();
    let (inventory_findings, inventory) = crate::sync::inventory::scan(instance_dir, &known).await;
    findings.extend(inventory_findings);
    inventory.save(instance_dir).await;

    cache.save(instance_dir).await;

    IntegrityReport {
        server_id: manifest.server_id,
        build_id: manifest.build_id,
        build_version: manifest.version.clone(),
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
        enabled_optional: enabled_optional.to_vec(),
        findings,
        checked_files: checked,
        block_launch,
    }
}

fn finding(
    kind: IntegrityKind,
    subject: &str,
    detail: Option<String>,
    repaired: bool,
) -> IntegrityFinding {
    IntegrityFinding {
        kind,
        subject: subject.to_string(),
        detail,
        repaired,
    }
}

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;
