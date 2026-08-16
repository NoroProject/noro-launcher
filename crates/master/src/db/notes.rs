//! Внутренние заметки на карточке игрока и журнал запусков.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NoteRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub author_id: Option<Uuid>,
    pub author_label: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

pub async fn add_note(
    pool: &PgPool,
    user_id: Uuid,
    author_id: Option<Uuid>,
    author_label: &str,
    body: &str,
) -> Result<NoteRow> {
    Ok(sqlx::query_as::<_, NoteRow>(
        "INSERT INTO user_notes (user_id, author_id, author_label, body)
         VALUES ($1,$2,$3,$4) RETURNING *",
    )
    .bind(user_id)
    .bind(author_id)
    .bind(author_label)
    .bind(body)
    .fetch_one(pool)
    .await?)
}

pub async fn list_notes(pool: &PgPool, user_id: Uuid) -> Result<Vec<NoteRow>> {
    Ok(sqlx::query_as::<_, NoteRow>(
        "SELECT * FROM user_notes WHERE user_id = $1 ORDER BY created_at DESC LIMIT 200",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn delete_note(pool: &PgPool, id: Uuid) -> Result<bool> {
    Ok(sqlx::query("DELETE FROM user_notes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected()
        > 0)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PlaySessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub server_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub playtime_secs: Option<i64>,
    pub build_version: String,
    pub launcher_version: String,
    pub enabled_optional: Value,
    pub integrity_ok: Option<bool>,
}

/// Открыть запись о запуске со снапшотом того, с чем игрок зашёл.
///
/// Раньше хранился только playtime, и разобрать задним числом «с какой сборкой
/// и какими модами он играл» было невозможно.
pub async fn open_play_session(
    pool: &PgPool,
    user_id: Uuid,
    server_id: Uuid,
    build_version: &str,
    launcher_version: &str,
    enabled_optional: &Value,
    integrity_ok: Option<bool>,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO play_sessions
           (user_id, server_id, build_version, launcher_version, enabled_optional, integrity_ok)
         VALUES ($1,$2,$3,$4,$5,$6)
         RETURNING id",
    )
    .bind(user_id)
    .bind(server_id)
    .bind(build_version)
    .bind(launcher_version)
    .bind(enabled_optional)
    .bind(integrity_ok)
    .fetch_one(pool)
    .await?)
}

/// Закрыть последнюю открытую запись.
pub async fn close_play_session(
    pool: &PgPool,
    user_id: Uuid,
    server_id: Uuid,
    playtime_secs: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE play_sessions SET ended_at = NOW(), playtime_secs = $3
         WHERE id = (
             SELECT id FROM play_sessions
             WHERE user_id = $1 AND server_id = $2 AND ended_at IS NULL
             ORDER BY started_at DESC LIMIT 1
         )",
    )
    .bind(user_id)
    .bind(server_id)
    .bind(playtime_secs)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_play_sessions(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<PlaySessionRow>> {
    Ok(sqlx::query_as::<_, PlaySessionRow>(
        "SELECT * FROM play_sessions WHERE user_id = $1 ORDER BY started_at DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?)
}
