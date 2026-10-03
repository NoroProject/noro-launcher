//! What the launcher needs to start an installed build without the master.
//!
//! The server list, the player's profile and each installed build's manifest
//! used to live only in memory, so with the master unreachable an installed
//! build couldn't be launched at all: no list to pick it from, no manifest to
//! launch it with, and the window stuck on the sign-in screen.
//!
//! Manifests are kept as received, signature included, and checked again when
//! read back: a file in a folder anyone running as the player can write to
//! decides nothing on its own.

use crate::directories::LauncherDirectories;
use schema::{BuildManifest, ServerEntry, UserProfile};
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn cache_dir(dirs: &LauncherDirectories) -> PathBuf {
    dirs.root().join("cache")
}

fn servers_path(dirs: &LauncherDirectories) -> PathBuf {
    cache_dir(dirs).join("servers.json")
}

fn profile_path(dirs: &LauncherDirectories) -> PathBuf {
    cache_dir(dirs).join("profile.json")
}

/// Beside the instance it describes, so deleting the instance takes it along.
fn manifest_path(dirs: &LauncherDirectories, server_id: &Uuid) -> PathBuf {
    dirs.instance(server_id).join(".noro").join("manifest.json")
}

fn read<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = std::fs::read(path).ok()?;
    match serde_json::from_slice(&bytes) {
        Ok(value) => Some(value),
        Err(e) => {
            tracing::warn!("ignoring unreadable {}: {e}", path.display());
            None
        }
    }
}

/// In the background: these are conveniences, and the loop that calls this
/// shouldn't wait on the disk for them.
fn write<T: Serialize>(path: PathBuf, value: &T) {
    let Ok(bytes) = serde_json::to_vec(value) else {
        return;
    };
    tokio::spawn(async move {
        if let Err(e) = crate::fsutil::write_atomic(&path, bytes).await {
            tracing::warn!("could not cache {}: {e}", path.display());
        }
    });
}

pub fn load_servers(dirs: &LauncherDirectories) -> Vec<ServerEntry> {
    read(&servers_path(dirs)).unwrap_or_default()
}

pub fn save_servers(dirs: &LauncherDirectories, servers: &[ServerEntry]) {
    write(servers_path(dirs), &servers);
}

pub fn load_profile(dirs: &LauncherDirectories) -> Option<UserProfile> {
    read(&profile_path(dirs))
}

pub fn save_profile(dirs: &LauncherDirectories, user: &UserProfile) {
    write(profile_path(dirs), user);
}

/// On sign-out: the next person at this computer shouldn't start with the
/// previous player's name, nor see servers only their roles could.
pub fn forget_account(dirs: &LauncherDirectories) {
    let _ = std::fs::remove_file(profile_path(dirs));
    let _ = std::fs::remove_file(servers_path(dirs));
}

/// Only for builds that are on disk; a manifest for a build that was never
/// installed has nothing to launch.
pub fn save_manifest(dirs: &LauncherDirectories, manifest: &BuildManifest) {
    if !dirs.instance(&manifest.server_id).is_dir() {
        return;
    }
    write(manifest_path(dirs, &manifest.server_id), manifest);
}

pub fn load_manifest(dirs: &LauncherDirectories, server_id: &Uuid) -> Option<BuildManifest> {
    let manifest: BuildManifest = read(&manifest_path(dirs, server_id))?;
    if manifest.server_id != *server_id || !crate::signing::verify_manifest(&manifest) {
        tracing::warn!(%server_id, "cached manifest failed its signature check, ignoring it");
        return None;
    }
    Some(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_http::TempDir;

    fn dirs(root: &Path) -> LauncherDirectories {
        LauncherDirectories {
            root: root.to_path_buf(),
        }
    }

    fn build(server_id: Uuid) -> BuildManifest {
        BuildManifest {
            server_id,
            ..crate::sync::verify::fixtures::manifest(Vec::new())
        }
    }

    async fn settle() {
        // The writes run on spawned tasks.
        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    #[tokio::test]
    async fn a_manifest_that_was_tampered_with_is_ignored() {
        use ed25519_dalek::Signer;
        let tmp = TempDir::new("offline-cache");
        let dirs = dirs(tmp.path());
        let server_id = Uuid::new_v4();
        std::fs::create_dir_all(dirs.instance(&server_id)).unwrap();

        let mut manifest = build(server_id);
        let key = ed25519_dalek::SigningKey::from_bytes(&schema::DEV_SIGNING_SEED);
        manifest.signature = key.sign(&manifest.signing_bytes()).to_bytes().to_vec();

        save_manifest(&dirs, &manifest);
        settle().await;
        assert_eq!(load_manifest(&dirs, &server_id), Some(manifest.clone()));

        manifest.main_class = "evil.Main".into();
        std::fs::write(
            manifest_path(&dirs, &server_id),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(load_manifest(&dirs, &server_id), None);
    }

    #[tokio::test]
    async fn nothing_is_cached_for_a_build_that_isnt_installed() {
        let tmp = TempDir::new("offline-cache");
        let dirs = dirs(tmp.path());
        let server_id = Uuid::new_v4();
        let manifest = build(server_id);
        save_manifest(&dirs, &manifest);
        settle().await;
        assert!(!dirs.instance(&server_id).exists());
    }

    #[tokio::test]
    async fn sign_out_forgets_the_account() {
        let tmp = TempDir::new("offline-cache");
        let dirs = dirs(tmp.path());
        save_servers(&dirs, &[]);
        settle().await;
        assert!(servers_path(&dirs).exists());
        forget_account(&dirs);
        assert!(!servers_path(&dirs).exists());
        assert!(load_profile(&dirs).is_none());
    }
}
