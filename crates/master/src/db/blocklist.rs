//! Запрещённые файлы.

use anyhow::Result;
use chrono::{DateTime, Utc};
use schema::{BlockAction, BlockedFile};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BlockedFileRow {
    pub id: Uuid,
    pub pattern: Option<String>,
    pub sha1: Option<String>,
    pub reason: String,
    pub action: String,
    pub server_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

fn action_from(s: &str) -> BlockAction {
    match s {
        "flag" => BlockAction::Flag,
        "block_launch" => BlockAction::BlockLaunch,
        _ => BlockAction::Delete,
    }
}

/// Правила для манифеста: глобальные плюс относящиеся к этому серверу.
pub async fn blocked_files_for(pool: &PgPool, server_id: Uuid) -> Result<Vec<BlockedFile>> {
    let rows = sqlx::query_as::<_, BlockedFileRow>(
        "SELECT * FROM blocked_files
         WHERE server_id IS NULL OR server_id = $1
         ORDER BY sort_order, created_at",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| BlockedFile {
            pattern: r.pattern,
            sha1: r.sha1,
            reason: r.reason,
            action: action_from(&r.action),
        })
        .collect())
}

pub async fn list_blocked_files(pool: &PgPool) -> Result<Vec<BlockedFileRow>> {
    Ok(sqlx::query_as::<_, BlockedFileRow>(
        "SELECT * FROM blocked_files ORDER BY sort_order, created_at",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_blocked_file(
    pool: &PgPool,
    pattern: Option<&str>,
    sha1: Option<&str>,
    reason: &str,
    action: &str,
    server_id: Option<Uuid>,
    by: Option<Uuid>,
) -> Result<BlockedFileRow> {
    Ok(sqlx::query_as::<_, BlockedFileRow>(
        "INSERT INTO blocked_files (pattern, sha1, reason, action, server_id, created_by)
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING *",
    )
    .bind(pattern)
    .bind(sha1)
    .bind(reason)
    .bind(action)
    .bind(server_id)
    .bind(by)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_blocked_file(pool: &PgPool, id: Uuid) -> Result<bool> {
    Ok(sqlx::query("DELETE FROM blocked_files WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected()
        > 0)
}
