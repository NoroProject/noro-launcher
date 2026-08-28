//! Главное требование — не завалить админку флагами: первый проход молчит, а
//! файлы сборки не считаются находками вообще.

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
    // Иначе при раскатке админка получила бы флаг на каждый файл каждого
    // игрока разом.
    let dir = Scratch::new("first");
    dir.write("resourcepacks/мой-пак.zip", "содержимое");

    let (findings, inv) = scan(dir.path(), &[]).await;

    assert!(findings.is_empty());
    inv.save(dir.path()).await;
}

#[tokio::test]
async fn a_new_file_is_flagged_on_the_second_pass() {
    let dir = Scratch::new("new");
    dir.write("resourcepacks/старый.zip", "было");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    dir.write("resourcepacks/новый.zip", "появилось");
    let (findings, _) = scan(dir.path(), &[]).await;

    assert_eq!(subjects(&findings), ["resourcepacks/новый.zip"]);
    assert!(!findings[0].repaired);
}

#[tokio::test]
async fn a_changed_file_is_flagged() {
    let dir = Scratch::new("changed");
    dir.write("config/мой.cfg", "было");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    dir.write("config/мой.cfg", "стало другим");
    let (findings, _) = scan(dir.path(), &[]).await;

    assert_eq!(subjects(&findings), ["config/мой.cfg"]);
    assert_eq!(findings[0].detail.as_deref(), Some("изменён"));
}

#[tokio::test]
async fn an_unchanged_file_says_nothing() {
    let dir = Scratch::new("same");
    dir.write("config/мой.cfg", "не меняется");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    let (findings, _) = scan(dir.path(), &[]).await;
    assert!(findings.is_empty());
}

#[tokio::test]
async fn build_files_are_not_findings() {
    // Их целостность проверяется отдельно; дублировать флаг незачем.
    let dir = Scratch::new("known");
    dir.write("mods/core.jar", "файл сборки");
    let (_, inv) = scan(dir.path(), &["mods/core.jar".to_string()]).await;
    inv.save(dir.path()).await;

    dir.write("mods/core.jar", "подменён");
    let (findings, _) = scan(dir.path(), &["mods/core.jar".to_string()]).await;

    assert!(findings.is_empty());
}

#[tokio::test]
async fn worlds_are_never_walked() {
    // saves/ это гигабайты, и хешировать их ради инвентаря нельзя.
    let dir = Scratch::new("saves");
    dir.write("saves/Мир/level.dat", "мир");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    dir.write("saves/Мир/level.dat", "мир изменился");
    let (findings, _) = scan(dir.path(), &[]).await;

    assert!(findings.is_empty());
}

#[tokio::test]
async fn screenshots_and_logs_are_ignored() {
    let dir = Scratch::new("noise");
    let (_, inv) = scan(dir.path(), &[]).await;
    inv.save(dir.path()).await;

    dir.write("screenshots/2026.png", "картинка");
    dir.write("logs/latest.log", "лог");
    let (findings, _) = scan(dir.path(), &[]).await;

    assert!(findings.is_empty(), "{:?}", subjects(&findings));
}
