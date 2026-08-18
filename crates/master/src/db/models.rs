//! Row-структуры, мапящиеся из таблиц через sqlx::FromRow.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserRow {
    pub id: Uuid,
    pub discord_id: Option<String>,
    pub discord_username: Option<String>,
    pub discord_avatar: Option<String>,
    pub mc_uuid: Uuid,
    pub mc_username: String,
    pub skin_url: Option<String>,
    /// Тонкая модель (Алекс). `false` — классическая (Стив): именно её клиент
    /// подразумевает, когда метаданных у текстуры нет.
    pub skin_slim: bool,
    pub cape_url: Option<String>,
    pub banned: bool,
    pub ban_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub is_local_account: bool,
    pub can_play: bool,
    pub is_root: bool,
    /// Выбранный язык. `None` — не выбирал: тогда работает язык по умолчанию.
    /// Нужен там, где языка клиента ещё нет, — прежде всего на экране отказа
    /// при входе в игру.
    pub locale: Option<String>,
    #[sqlx(default)]
    pub hide_from_online: bool,
    #[sqlx(default)]
    pub silent_join: bool,
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
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub parent_id: Option<Uuid>,
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
    // Колонка NOT NULL DEFAULT TRUE (миграция 0017) — значение всегда приходит из БД.
    pub allow_optional_mod_suggestions: bool,
    pub recommended_memory_min_mb: i32,
    pub recommended_memory_max_mb: i32,
    pub recommended_jvm_flags: String,
    pub recommended_show_console_on_launch: bool,
    pub unmanaged_paths: serde_json::Value,
    pub user_managed_paths: serde_json::Value,
    pub path_rules: Option<serde_json::Value>,
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

/// Строка admin-токена. Не `Serialize`: и селектор, и хеш — служебные значения,
/// которым нечего делать в ответе API. Наружу их отдавал прежний список
/// токенов, где `token_hash` был ещё и рабочим доказательством владения.
#[derive(Debug, Clone, FromRow)]
pub struct AdminTokenRow {
    pub id: Uuid,
    pub name: String,
    /// SHA-256 секрета: по нему ищется строка.
    pub token_lookup: String,
    /// argon2 в формате PHC. `None` — токен достался от старой схемы и будет
    /// переведён при первом же использовании.
    pub token_hash: Option<String>,
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
    /// Платформа файла ("windows-x86_64"); NULL — нужен всем.
    pub platform: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ModSuggestionRow {
    pub id: Uuid,
    pub server_id: Uuid,
    pub build_id: Option<Uuid>,
    pub provider: String,
    pub project_id: String,
    pub title: String,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub suggested_by: Uuid,
    /// Ник игрока из users. Заполняет только список заявок — остальным запросам
    /// имя не нужно, поэтому поле необязательное.
    #[sqlx(default)]
    pub suggested_by_name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
