//! Mostly about not burying the admin panel in findings: the first pass is
//! silent, and build files never count.

use super::*;
use std::path::PathBuf;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("noro-inv-{name}-{}", std::process::id()));
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

fn subjects(f: &[IntegrityFinding]) -> Vec<&str> {
    f.iter().map(|x| x.subject.as_str()).collect()
}

#[tokio::test]
async fn the_first_pass_only_remembers() {
    // Otherwise the rollout flags every file of every player at once.
    let dir = Scratch::new("first");
    dir.write("resourcepacks/my-pack.zip", "contents");

    let (findings, inv) = scan(dir.path(), &[]).await;

    assert!(findings.is_empty());
    inv.save(dir.path()).await;
}

#[tokio::test]
async fn a_new_file_is_flagged_on_the_second_pass() {
    let dir = Scratch::new("new");
    dir.write("resourcepacks/old.zip", "was here");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    dir.write("resourcepacks/new.zip", "showed up");
    let (findings, _) = scan(dir.path(), &[]).await;

    assert_eq!(subjects(&findings), ["resourcepacks/new.zip"]);
    assert!(!findings[0].repaired);
}

#[tokio::test]
async fn a_changed_file_is_flagged() {
    let dir = Scratch::new("changed");
    dir.write("config/mine.cfg", "was here");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    dir.write("config/mine.cfg", "now different");
    let (findings, _) = scan(dir.path(), &[]).await;

    assert_eq!(subjects(&findings), ["config/mine.cfg"]);
    assert_eq!(findings[0].detail.as_deref(), Some("changed"));
}

#[tokio::test]
async fn an_unchanged_file_says_nothing() {
    let dir = Scratch::new("same");
    dir.write("config/mine.cfg", "never changes");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    let (findings, _) = scan(dir.path(), &[]).await;
    assert!(findings.is_empty());
}

#[tokio::test]
async fn build_files_are_not_findings() {
    // They have their own integrity check; no point flagging twice.
    let dir = Scratch::new("known");
    dir.write("mods/core.jar", "build file");
    let (_, inv) = scan(dir.path(), &["mods/core.jar".to_string()]).await;
    inv.save(dir.path()).await;

    dir.write("mods/core.jar", "swapped out");
    let (findings, _) = scan(dir.path(), &["mods/core.jar".to_string()]).await;

    assert!(findings.is_empty());
}

#[tokio::test]
async fn worlds_are_never_walked() {
    // saves/ is gigabytes; hashing it for an inventory is not on.
    let dir = Scratch::new("saves");
    dir.write("saves/World/level.dat", "a world");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    dir.write("saves/World/level.dat", "the world moved on");
    let (findings, _) = scan(dir.path(), &[]).await;

    assert!(findings.is_empty());
}

#[tokio::test]
async fn screenshots_and_logs_are_ignored() {
    let dir = Scratch::new("noise");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    dir.write("screenshots/2026.png", "a picture");
    dir.write("logs/latest.log", "a log");
    let (findings, _) = scan(dir.path(), &[]).await;

    assert!(findings.is_empty(), "{:?}", subjects(&findings));
}
