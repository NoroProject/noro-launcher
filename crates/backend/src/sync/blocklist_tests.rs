//! The point of the list is that it reaches files sync never looks at, so
//! that is what these check first.

use super::*;
use std::path::PathBuf;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("noro-block-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
    fn write(&self, rel: &str, body: &str) {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn rule(pattern: &str, action: BlockAction) -> BlockedFile {
    BlockedFile {
        pattern: Some(pattern.into()),
        sha1: None,
        reason: "xray".into(),
        action,
    }
}

#[tokio::test]
async fn a_banned_file_inside_an_unsynced_folder_is_still_removed() {
    // The whole reason the list exists: resourcepacks/ is never synced, but
    // an xray pack in it still gets deleted.
    let dir = Scratch::new("unsynced");
    dir.write("resourcepacks/super-xray.zip", "blocked");
    dir.write("resourcepacks/plain.zip", "fine");

    let report = enforce(dir.path(), &[rule("*xray*", BlockAction::Delete)]).await;

    assert_eq!(report.findings.len(), 1);
    assert!(report.findings[0].repaired);
    assert!(!dir.path().join("resourcepacks/super-xray.zip").exists());
    assert!(dir.path().join("resourcepacks/plain.zip").exists());
}

#[tokio::test]
async fn flag_reports_without_deleting() {
    let dir = Scratch::new("flag");
    dir.write("mods/suspicious.jar", "contents");

    let report = enforce(dir.path(), &[rule("*suspicious*", BlockAction::Flag)]).await;

    assert_eq!(report.findings.len(), 1);
    assert!(!report.findings[0].repaired);
    assert!(dir.path().join("mods/suspicious.jar").exists());
    assert!(!report.block_launch);
}

#[tokio::test]
async fn block_launch_stops_the_game() {
    let dir = Scratch::new("block");
    dir.write("mods/cheat.jar", "contents");

    let report = enforce(dir.path(), &[rule("*cheat*", BlockAction::BlockLaunch)]).await;

    assert!(report.block_launch);
    // Kept on disk so the player can see what is holding the launch.
    assert!(dir.path().join("mods/cheat.jar").exists());
}

#[tokio::test]
async fn a_hash_rule_catches_a_renamed_file() {
    let dir = Scratch::new("hash");
    dir.write("resourcepacks/harmless.zip", "blocked contents");
    let sha1 = crate::sync::integrity::sha1_file(&dir.path().join("resourcepacks/harmless.zip"))
        .await
        .unwrap();

    let report = enforce(
        dir.path(),
        &[BlockedFile {
            pattern: None,
            sha1: Some(sha1),
            reason: "known xray build".into(),
            action: BlockAction::Delete,
        }],
    )
    .await;

    assert_eq!(report.findings.len(), 1);
    assert!(!dir.path().join("resourcepacks/harmless.zip").exists());
}

#[tokio::test]
async fn saves_are_never_scanned() {
    // Gigabytes of world data, and no blocked file ever lives there.
    let dir = Scratch::new("saves");
    dir.write("saves/World/xray-data.dat", "anything");

    let report = enforce(dir.path(), &[rule("*xray*", BlockAction::Delete)]).await;

    assert!(report.findings.is_empty());
    assert!(dir.path().join("saves/World/xray-data.dat").exists());
}

#[tokio::test]
async fn the_launcher_service_directory_is_left_alone() {
    let dir = Scratch::new("service");
    dir.write(".noro/base-hashes.json", "{}");

    let report = enforce(dir.path(), &[rule("*", BlockAction::Delete)]).await;

    assert!(report.findings.is_empty());
    assert!(dir.path().join(".noro/base-hashes.json").exists());
}

#[tokio::test]
async fn an_empty_ruleset_does_no_work() {
    let dir = Scratch::new("empty");
    dir.write("mods/xray.jar", "contents");

    let report = enforce(dir.path(), &[]).await;

    assert!(report.findings.is_empty());
    assert!(dir.path().join("mods/xray.jar").exists());
}
