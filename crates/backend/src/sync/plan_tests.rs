//! Проверяется то, ради чего появился режим: обновление доезжает, правки не
//! затираются, а конфликт разрешается по политике.

use super::*;
use schema::{ArtifactKind, FileSide, Modloader, PathRule, RecommendedClientSettings};
use std::collections::BTreeMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Свой каталог на тест: план читает диск.
struct Scratch(PathBuf);

impl Scratch {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(name: &str) -> Scratch {
    let dir = std::env::temp_dir().join(format!("noro-plan-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    Scratch(dir)
}

/// Манифест с одним файлом `config/a.json` и одним правилом.
fn merged_manifest(pattern: &str, server_sha1: &str) -> BuildManifest {
    BuildManifest {
        build_id: Uuid::nil(),
        server_id: Uuid::nil(),
        version: "1".into(),
        mc_version: "1.20.1".into(),
        modloader: Modloader::Fabric,
        modloader_version: None,
        main_class: "Main".into(),
        jvm_args: Vec::new(),
        game_args: Vec::new(),
        assets_index_name: "1.20".into(),
        verified_files: vec![FileEntry {
            path: "config/a.json".into(),
            sha1: server_sha1.into(),
            size: 1,
            url: String::new(),
            side: FileSide::Both,
            executable: false,
            platform: None,
        }],
        artifact_kinds: BTreeMap::from([("config/a.json".to_string(), ArtifactKind::Config)]),
        unmanaged_paths: Vec::new(),
        user_managed_paths: Vec::new(),
        path_rules: vec![PathRule {
            pattern: pattern.into(),
            mode: schema::PathMode::Merged,
            conflict: schema::ConflictPolicy::KeepMine,
        }],
        blocked_files: Vec::new(),
        optional_mods: Vec::new(),
        allow_optional_mod_suggestions: false,
        recommended_client_settings: RecommendedClientSettings {
            memory_min_mb: 512,
            memory_max_mb: 2048,
            jvm_flags: String::new(),
            show_console_on_launch: false,
            fullscreen: false,
        },
        signature: Vec::new(),
    }
}

/// Файл на диске + его настоящий sha1.
async fn write(dir: &std::path::Path, rel: &str, body: &str) -> String {
    let path = dir.join(rel);
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&path, body).await.unwrap();
    crate::sync::integrity::sha1_file(&path).await.unwrap()
}

fn action_name(a: &Action) -> &'static str {
    match a {
        Action::Download => "download",
        Action::Skip => "skip",
        Action::Conflict(_) => "conflict",
    }
}

#[tokio::test]
async fn merged_updates_a_file_the_player_never_touched() {
    // Ровно тот случай, ради которого режим и появился: user_managed оставлял
    // игрока на старом конфиге навсегда.
    let dir = scratch("merged-update");
    let mine = write(dir.path(), "config/a.json", "старое").await;
    let m = merged_manifest("config/**", "новыйsha1");
    let mut base = BaseHashes::default();
    base.set("config/a.json", &mine);

    let a = decide_file(dir.path(), &m, &m.verified_files[0], &base, true).await;
    assert_eq!(action_name(&a), "download");
}

#[tokio::test]
async fn merged_keeps_edits_the_server_did_not_touch() {
    let dir = scratch("merged-keep");
    write(dir.path(), "config/a.json", "правки игрока").await;
    let server_sha1 = "серверный";
    let m = merged_manifest("config/**", server_sha1);
    let mut base = BaseHashes::default();
    // База совпадает с серверным — значит сервер ничего не менял.
    base.set("config/a.json", server_sha1);

    let a = decide_file(dir.path(), &m, &m.verified_files[0], &base, true).await;
    assert_eq!(action_name(&a), "skip");
}

#[tokio::test]
async fn merged_reports_a_conflict_when_both_sides_changed() {
    let dir = scratch("merged-conflict");
    write(dir.path(), "config/a.json", "правки игрока").await;
    let m = merged_manifest("config/**", "серверный");
    let mut base = BaseHashes::default();
    base.set("config/a.json", "исходный");

    let a = decide_file(dir.path(), &m, &m.verified_files[0], &base, true).await;
    assert_eq!(action_name(&a), "conflict");
}

#[tokio::test]
async fn a_missing_file_is_installed_in_every_mode() {
    let dir = scratch("missing");
    let m = merged_manifest("config/**", "серверный");
    let a = decide_file(
        dir.path(),
        &m,
        &m.verified_files[0],
        &BaseHashes::default(),
        true,
    )
    .await;
    assert_eq!(action_name(&a), "download");
}

#[tokio::test]
async fn user_managed_never_updates_an_existing_file() {
    let dir = scratch("user-managed");
    write(dir.path(), "config/a.json", "правки игрока").await;
    let mut m = merged_manifest("config/**", "серверный");
    m.path_rules[0].mode = schema::PathMode::UserManaged;

    let a = decide_file(
        dir.path(),
        &m,
        &m.verified_files[0],
        &BaseHashes::default(),
        true,
    )
    .await;
    assert_eq!(action_name(&a), "skip");
}

#[tokio::test]
async fn unmanaged_is_never_touched() {
    let dir = scratch("unmanaged");
    let mut m = merged_manifest("config/**", "серверный");
    m.path_rules[0].mode = schema::PathMode::Unmanaged;

    let a = decide_file(
        dir.path(),
        &m,
        &m.verified_files[0],
        &BaseHashes::default(),
        true,
    )
    .await;
    assert_eq!(action_name(&a), "skip");
}
