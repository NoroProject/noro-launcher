//! Тест закрывает дыру, ради которой фильтр и написан: файлы limited-мода не
//! должны доезжать до игрока без права, иначе решение «включать или нет»
//! принимает клиент, а он открытый.

use super::*;
use schema::{
    ArtifactKind, FileEntry, FileSide, Modloader, OptionalMod, RecommendedClientSettings,
};

use uuid::Uuid;

fn server_id() -> Uuid {
    Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
}

fn opt_mod(name: &str, limited: bool, files: &[&str]) -> OptionalMod {
    OptionalMod {
        name: name.into(),
        description: String::new(),
        category: "Геймплей".into(),
        files: files.iter().map(|s| s.to_string()).collect(),
        enabled_by_default: false,
        visible: true,
        limited,
        dependencies: Vec::new(),
        conflicts: Vec::new(),
        triggers: Vec::new(),
        os: Vec::new(),
        icon_url: None,
        author: None,
    }
}

fn file(path: &str) -> FileEntry {
    FileEntry {
        path: path.into(),
        sha1: "0".repeat(40),
        size: 1,
        url: format!("https://example.invalid/{path}"),
        side: FileSide::Both,
        executable: false,
        platform: None,
    }
}

fn viewer(permissions: &[&str]) -> UserProfile {
    UserProfile {
        id: Uuid::nil(),
        uuid: Uuid::nil(),
        username: "player".into(),
        discord_id: Some("0".into()),
        discord_username: Some("player".into()),
        discord_avatar: None,
        skin_url: None,
        skin_slim: false,
        cape_url: None,
        roles: Vec::new(),
        permissions: permissions.iter().map(|p| p.to_string()).collect(),
        permission_grants: Vec::new(),
        banned: false,
        is_local_account: false,
        can_play: true,
        is_root: false,
        hide_from_online: false,
        frozen: false,
        freeze_info: None,
        silent_join: false,
    }
}

fn paths(m: &BuildManifest) -> Vec<&str> {
    m.verified_files.iter().map(|f| f.path.as_str()).collect()
}

fn manifest(mods: Vec<OptionalMod>, paths: &[&str]) -> BuildManifest {
    BuildManifest {
        build_id: Uuid::nil(),
        server_id: server_id(),
        version: "1".into(),
        mc_version: "1.20.1".into(),
        modloader: Modloader::Fabric,
        modloader_version: None,
        main_class: "Main".into(),
        jvm_args: Vec::new(),
        game_args: Vec::new(),
        assets_index_name: "1.20".into(),
        verified_files: paths.iter().map(|p| file(p)).collect(),
        artifact_kinds: paths
            .iter()
            .map(|p| (p.to_string(), ArtifactKind::Mod))
            .collect(),
        unmanaged_paths: Vec::new(),
        path_rules: Vec::new(),
        blocked_files: Vec::new(),
        user_managed_paths: Vec::new(),
        optional_mods: mods,
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

#[test]
fn limited_mod_without_permission_loses_files() {
    let mut m = manifest(
        vec![
            opt_mod("staff", true, &["mods/staff.jar"]),
            opt_mod("optifine", false, &["mods/optifine.jar"]),
        ],
        &["mods/staff.jar", "mods/optifine.jar", "mods/core.jar"],
    );

    filter_for_viewer(&mut m, &viewer(&[]), "");

    assert_eq!(paths(&m), vec!["mods/optifine.jar", "mods/core.jar"]);
    assert_eq!(m.optional_mods.len(), 1);
    assert_eq!(m.optional_mods[0].name, "optifine");
    assert!(!m.artifact_kinds.contains_key("mods/staff.jar"));
}

#[test]
fn granted_permission_keeps_everything() {
    let perm = format!("noro.optional.{}.staff", server_id());
    let mut m = manifest(
        vec![opt_mod("staff", true, &["mods/staff.jar"])],
        &["mods/staff.jar", "mods/core.jar"],
    );

    filter_for_viewer(&mut m, &viewer(&[&perm]), "");

    assert_eq!(paths(&m), vec!["mods/staff.jar", "mods/core.jar"]);
    assert_eq!(m.optional_mods.len(), 1);
}

#[test]
fn file_shared_with_allowed_mod_survives() {
    let perm = format!("noro.optional.{}.allowed", server_id());
    let mut m = manifest(
        vec![
            opt_mod("denied", true, &["mods/shared.jar", "mods/denied.jar"]),
            opt_mod("allowed", true, &["mods/shared.jar"]),
        ],
        &["mods/shared.jar", "mods/denied.jar"],
    );

    filter_for_viewer(&mut m, &viewer(&[&perm]), "");

    assert_eq!(paths(&m), vec!["mods/shared.jar"]);
}

/// Мод для чужой системы уезжать не должен: его файлы попали бы в
/// `verified_files`, лаунчер скачал бы их, а сверка удаляла бы как лишние.
#[test]
fn mod_for_another_system_loses_files() {
    let mut windows_only = opt_mod("shaders", false, &["mods/shaders.jar"]);
    windows_only.os = vec!["windows".into()];
    let mut m = manifest(vec![windows_only], &["mods/shaders.jar", "mods/core.jar"]);

    filter_for_viewer(&mut m, &viewer(&[]), "macos");

    assert!(m.optional_mods.is_empty());
    assert_eq!(paths(&m), vec!["mods/core.jar"]);
}

/// На своей системе тот же мод остаётся целиком.
#[test]
fn mod_for_this_system_stays() {
    let mut windows_only = opt_mod("shaders", false, &["mods/shaders.jar"]);
    windows_only.os = vec!["windows".into()];
    let mut m = manifest(vec![windows_only], &["mods/shaders.jar"]);

    filter_for_viewer(&mut m, &viewer(&[]), "windows");

    assert_eq!(m.optional_mods.len(), 1);
    assert_eq!(paths(&m), vec!["mods/shaders.jar"]);
}

/// Лаунчеры, выпущенные до передачи платформы, шлют пустую строку. Спрятать от
/// них моды значило бы сломать работающие сборки.
#[test]
fn unknown_system_keeps_everything() {
    let mut windows_only = opt_mod("shaders", false, &["mods/shaders.jar"]);
    windows_only.os = vec!["windows".into()];
    let mut m = manifest(vec![windows_only], &["mods/shaders.jar"]);

    filter_for_viewer(&mut m, &viewer(&[]), "");

    assert_eq!(m.optional_mods.len(), 1);
}

/// Платформа приходит с архитектурой, а моды различаются только системой.
#[test]
fn platform_narrows_down_to_the_system() {
    assert_eq!(super::os_of("macos-aarch64"), "macos");
    assert_eq!(super::os_of("windows-x86_64"), "windows");
    assert_eq!(super::os_of(""), "");
}
