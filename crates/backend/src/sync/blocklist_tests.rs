//! Смысл базы в том, что она достаёт файл там, куда синк не ходит. Это и
//! проверяется в первую очередь.

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
    // Ровно то, ради чего база и нужна: resourcepacks/ не синхронизируется,
    // но xray оттуда удаляется.
    let dir = Scratch::new("unsynced");
    dir.write("resourcepacks/super-xray.zip", "запрещённое");
    dir.write("resourcepacks/обычный.zip", "нормальное");

    let report = enforce(dir.path(), &[rule("*xray*", BlockAction::Delete)]).await;

    assert_eq!(report.findings.len(), 1);
    assert!(report.findings[0].repaired);
    assert!(!dir.path().join("resourcepacks/super-xray.zip").exists());
    assert!(dir.path().join("resourcepacks/обычный.zip").exists());
}

#[tokio::test]
async fn flag_reports_without_deleting() {
    let dir = Scratch::new("flag");
    dir.write("mods/подозрительный.jar", "содержимое");

    let report = enforce(dir.path(), &[rule("*подозрительный*", BlockAction::Flag)]).await;

    assert_eq!(report.findings.len(), 1);
    assert!(!report.findings[0].repaired);
    assert!(dir.path().join("mods/подозрительный.jar").exists());
    assert!(!report.block_launch);
}

#[tokio::test]
async fn block_launch_stops_the_game() {
    let dir = Scratch::new("block");
    dir.write("mods/cheat.jar", "содержимое");

    let report = enforce(dir.path(), &[rule("*cheat*", BlockAction::BlockLaunch)]).await;

    assert!(report.block_launch);
    // Не удаляем: игрок должен увидеть, из-за чего его не пускают.
    assert!(dir.path().join("mods/cheat.jar").exists());
}

#[tokio::test]
async fn a_hash_rule_catches_a_renamed_file() {
    let dir = Scratch::new("hash");
    dir.write("resourcepacks/безобидное.zip", "запрещённое содержимое");
    let sha1 = crate::sync::integrity::sha1_file(&dir.path().join("resourcepacks/безобидное.zip"))
        .await
        .unwrap();

    let report = enforce(
        dir.path(),
        &[BlockedFile {
            pattern: None,
            sha1: Some(sha1),
            reason: "известная сборка xray".into(),
            action: BlockAction::Delete,
        }],
    )
    .await;

    assert_eq!(report.findings.len(), 1);
    assert!(!dir.path().join("resourcepacks/безобидное.zip").exists());
}

#[tokio::test]
async fn saves_are_never_scanned() {
    // Там гигабайты, а запрещённых файлов не бывает.
    let dir = Scratch::new("saves");
    dir.write("saves/Мир/xray-data.dat", "что угодно");

    let report = enforce(dir.path(), &[rule("*xray*", BlockAction::Delete)]).await;

    assert!(report.findings.is_empty());
    assert!(dir.path().join("saves/Мир/xray-data.dat").exists());
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
    dir.write("mods/xray.jar", "содержимое");

    let report = enforce(dir.path(), &[]).await;

    assert!(report.findings.is_empty());
    assert!(dir.path().join("mods/xray.jar").exists());
}
