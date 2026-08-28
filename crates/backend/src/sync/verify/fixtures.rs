//! Fixtures for the pre-launch verification tests.

use schema::{
    ArtifactKind, BuildManifest, FileEntry, FileSide, IntegrityKind, IntegrityReport, Modloader,
    OptionalMod, RecommendedClientSettings, UserProfile,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const SERVER: &str = "22222222-2222-2222-2222-222222222222";

/// sha1 of the body "ok".
pub const OK_SHA1: &str = "7a85f4764bbd6daf1c3545efbbf0f279a6dc0beb";

pub fn entry(path: &str, sha1: &str, kind: ArtifactKind) -> (FileEntry, (String, ArtifactKind)) {
    (
        FileEntry {
            path: path.into(),
            sha1: sha1.into(),
            size: 2,
            url: String::new(),
            side: FileSide::Both,
            executable: false,
            platform: None,
        },
        (path.to_string(), kind),
    )
}

pub fn manifest(files: Vec<(FileEntry, (String, ArtifactKind))>) -> BuildManifest {
    let mut kinds = BTreeMap::new();
    let mut verified = Vec::new();
    for (f, (path, kind)) in files {
        kinds.insert(path, kind);
        verified.push(f);
    }
    BuildManifest {
        build_id: Uuid::nil(),
        server_id: Uuid::parse_str(SERVER).unwrap(),
        version: "1".into(),
        mc_version: "1.20.1".into(),
        modloader: Modloader::Fabric,
        modloader_version: None,
        main_class: "Main".into(),
        jvm_args: Vec::new(),
        game_args: Vec::new(),
        assets_index_name: "1.20".into(),
        verified_files: verified,
        artifact_kinds: kinds,
        unmanaged_paths: Vec::new(),
        path_rules: Vec::new(),
        blocked_files: Vec::new(),
        user_managed_paths: Vec::new(),
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

pub fn optional(name: &str, limited: bool, files: &[&str]) -> OptionalMod {
    OptionalMod {
        name: name.into(),
        description: String::new(),
        category: "Gameplay".into(),
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

pub fn player() -> UserProfile {
    UserProfile {
        id: Uuid::nil(),
        uuid: Uuid::nil(),
        username: "player".into(),
        identities: Vec::new(),
        skin_url: None,
        skin_slim: false,
        cape_url: None,
        roles: Vec::new(),
        permissions: Vec::new(),
        permission_grants: Vec::new(),
        banned: false,
        ban_reason: None,
        created_at: None,
        last_login_at: None,
        is_local_account: false,
        can_play: true,
        is_root: false,
        hide_from_online: false,
        frozen: false,
        freeze_info: None,
        silent_join: false,
    }
}

/// One directory per test — verification deletes files, so a shared one would
/// make the tests depend on each other.
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("noro-verify-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub async fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(path, body).await.unwrap();
}

pub fn subjects(report: &IntegrityReport, kind: IntegrityKind) -> Vec<&str> {
    report
        .findings
        .iter()
        .filter(|f| f.kind == kind)
        .map(|f| f.subject.as_str())
        .collect()
}
