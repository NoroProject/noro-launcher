//! Какой лаунчер у какого игрока: версия, платформа, когда последний раз выходил на связь.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Строка для админки: игрок и его клиент.
#[derive(serde::Serialize, sqlx::FromRow)]
pub struct LauncherClientRow {
    pub user_id: Uuid,
    pub mc_username: String,
    pub version: String,
    pub platform: String,
    pub last_seen_at: DateTime<Utc>,
}

/// Запомнить клиент при авторизации по WebSocket.
///
/// История не хранится — важно текущее состояние, поэтому одна строка на игрока.
pub async fn record_launcher_client(
    pool: &PgPool,
    user_id: Uuid,
    version: &str,
    platform: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO launcher_clients (user_id, version, platform, last_seen_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id) DO UPDATE
         SET version = EXCLUDED.version,
             platform = EXCLUDED.platform,
             last_seen_at = NOW()",
    )
    .bind(user_id)
    .bind(version)
    .bind(platform)
    .execute(pool)
    .await?;
    Ok(())
}

/// Версия, которая сейчас раздаётся, безотносительно платформы.
///
/// `current_launcher_version` ищет по платформе, но клиент сообщает её как
/// `macos-aarch64`, а в `launcher_versions` лежит target-триплет. Релиз всё
/// равно выкатывается одной версией на все платформы, поэтому сравнивать проще
/// по ней одной.
pub async fn current_launcher_version_any(pool: &PgPool) -> Result<Option<String>> {
    Ok(sqlx::query_scalar(
        "SELECT version FROM launcher_versions
         WHERE is_current = TRUE AND kind = 'core' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?)
}

/// Клиент одного игрока — для карточки в админке.
pub async fn launcher_client(pool: &PgPool, user_id: Uuid) -> Result<Option<LauncherClientRow>> {
    Ok(sqlx::query_as::<_, LauncherClientRow>(
        "SELECT c.user_id, u.mc_username, c.version, c.platform, c.last_seen_at
         FROM launcher_clients c
         JOIN users u ON u.id = c.user_id
         WHERE c.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}

/// Все известные клиенты, свежие сверху.
pub async fn list_launcher_clients(pool: &PgPool) -> Result<Vec<LauncherClientRow>> {
    Ok(sqlx::query_as::<_, LauncherClientRow>(
        "SELECT c.user_id, u.mc_username, c.version, c.platform, c.last_seen_at
         FROM launcher_clients c
         JOIN users u ON u.id = c.user_id
         ORDER BY c.last_seen_at DESC
         LIMIT 500",
    )
    .fetch_all(pool)
    .await?)
}
