//! Row-структуры, мапящиеся из таблиц через sqlx::FromRow.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserRow {
    pub id: Uuid,
    pub discord_id: String,
    pub discord_username: String,
    pub discord_avatar: Option<String>,
    pub mc_uuid: Uuid,
    pub mc_username: String,
    pub skin_url: Option<String>,
    pub cape_url: Option<String>,
    pub banned: bool,
    pub ban_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RoleRow {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub color: Option<String>,
    pub is_default: bool,
    pub sort_order: i32,
    pub lp_group: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ServerRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub background_url: Option<String>,
    pub modloader: String,
    pub mc_version: String,
    pub active: bool,
    pub limited: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BuildRow {
    pub id: Uuid,
    pub server_id: Uuid,
    pub version: String,
    pub changelog: String,
    pub modloader: String,
    pub modloader_version: Option<String>,
    pub mc_version: String,
    pub main_class: String,
    pub jvm_args: serde_json::Value,
    pub game_args: serde_json::Value,
    pub assets_index_name: String,
    pub published: bool,
    pub optional_mods: serde_json::Value,
    pub recommended_memory_min_mb: i32,
    pub recommended_memory_max_mb: i32,
    pub recommended_jvm_flags: String,
    pub recommended_show_console_on_launch: bool,
    pub unmanaged_paths: serde_json::Value,
    pub user_managed_paths: serde_json::Value,
    pub manifest_signature: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BuildFileRow {
    pub id: Uuid,
    pub build_id: Uuid,
    pub path: String,
    pub sha1: String,
    pub size: i64,
    pub side: String,
    pub kind: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct MojangArtifactRow {
    pub id: Uuid,
    pub artifact_type: String,
    pub mc_version: Option<String>,
    pub platform: Option<String>,
    pub path: String,
    pub sha1: String,
    pub size: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct NewsRow {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub preview_img_url: Option<String>,
    pub author_id: Option<Uuid>,
    pub pinned: bool,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ServerCoreRow {
    pub id: Uuid,
    pub server_id: Uuid,
    pub version: String,
    pub sha256: String,
    pub file_sha1: String,
    pub size: i64,
    pub active: bool,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct LauncherVersionRow {
    pub id: Uuid,
    pub version: String,
    pub platform: String,
    pub sha256: String,
    pub file_sha1: String,
    pub size: i64,
    pub signature: String,
    pub is_current: bool,
    pub built_at: DateTime<Utc>,
    /// `core` — то, что качает bootstrapper; `bootstrapper` — то, что качает игрок.
    pub kind: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AdminTokenRow {
    pub id: Uuid,
    pub name: String,
    pub token_hash: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct LauncherBuildJobRow {
    pub id: Uuid,
    pub github_tag: String,
    pub status: String,
    pub log: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BaseBuildRow {
    pub id: Uuid,
    pub mc_version: String,
    pub modloader: String,
    pub modloader_version: Option<String>,
    pub main_class: String,
    pub jvm_args: serde_json::Value,
    pub game_args: serde_json::Value,
    pub assets_index_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BaseBuildFileRow {
    pub id: Uuid,
    pub base_build_id: Uuid,
    pub path: String,
    pub sha1: String,
    pub size: i64,
    pub side: String,
    pub kind: String,
}
