//! Бандлы логов: метаданные, сами архивы лежат в FileStore.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// Сколько живёт бандл. Дальше архив бесполезен, а хранить чужие логи вечно
/// незачем — осиротевший blob подберёт `files/gc.rs`.
pub const RETENTION_DAYS: i64 = 30;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SupportBundleRow {
    pub id: Uuid,
    pub at: DateTime<Utc>,
    pub user_id: Uuid,
    pub server_id: Option<Uuid>,
    pub note: String,
    pub voluntary: bool,
    pub file_sha1: String,
    pub size: i64,
    pub expires_at: DateTime<Utc>,
}

pub async fn create_support_bundle(
    pool: &PgPool,
    user_id: Uuid,
    server_id: Option<Uuid>,
    note: &str,
    voluntary: bool,
    file_sha1: &str,
    size: i64,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO support_bundles
           (user_id, server_id, note, voluntary, file_sha1, size, expires_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         RETURNING id",
    )
    .bind(user_id)
    .bind(server_id)
    .bind(note)
    .bind(voluntary)
    .bind(file_sha1)
    .bind(size)
    .bind(Utc::now() + Duration::days(RETENTION_DAYS))
    .fetch_one(pool)
    .await?)
}

pub async fn list_support_bundles(
    pool: &PgPool,
    user_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<SupportBundleRow>> {
    Ok(sqlx::query_as::<_, SupportBundleRow>(
        "SELECT * FROM support_bundles
         WHERE ($1::uuid IS NULL OR user_id = $1)
         ORDER BY at DESC
         LIMIT $2",
    )
    .bind(user_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?)
}

pub async fn get_support_bundle(pool: &PgPool, id: Uuid) -> Result<Option<SupportBundleRow>> {
    Ok(
        sqlx::query_as::<_, SupportBundleRow>("SELECT * FROM support_bundles WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Удалить бандл. `only_voluntary` ограничивает игрока: собранное по
/// админскому запросу он снести не может, иначе принудительный режим
/// бессмысленен.
pub async fn delete_support_bundle(
    pool: &PgPool,
    id: Uuid,
    owner: Option<Uuid>,
    only_voluntary: bool,
) -> Result<bool> {
    let res = sqlx::query(
        "DELETE FROM support_bundles
         WHERE id = $1
           AND ($2::uuid IS NULL OR user_id = $2)
           AND ($3 = FALSE OR voluntary = TRUE)",
    )
    .bind(id)
    .bind(owner)
    .bind(only_voluntary)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Убрать протухшие. Blob останется осиротевшим и уйдёт при сборке мусора.
pub async fn purge_expired_bundles(pool: &PgPool) -> Result<u64> {
    Ok(
        sqlx::query("DELETE FROM support_bundles WHERE expires_at < NOW()")
            .execute(pool)
            .await?
            .rows_affected(),
    )
}
