//! Cape catalog and user assignment queries.

use anyhow::Result;
use chrono::{DateTime, Utc};
use schema::CapeRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct CapeDbRow {
    id: Uuid,
    name: String,
    url: String,
    file_sha1: String,
    size: i64,
    uploaded_by: Option<Uuid>,
    uploaded_at: DateTime<Utc>,
}

impl From<CapeDbRow> for CapeRow {
    fn from(row: CapeDbRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            url: row.url,
            file_sha1: row.file_sha1,
            size: row.size,
            uploaded_by: row.uploaded_by,
            uploaded_at: row.uploaded_at,
        }
    }
}

pub async fn list_capes(pool: &PgPool) -> Result<Vec<CapeRow>> {
    let rows = sqlx::query_as::<_, CapeDbRow>("SELECT * FROM capes ORDER BY uploaded_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn list_capes_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<CapeRow>> {
    let rows = sqlx::query_as::<_, CapeDbRow>(
        "SELECT c.* FROM capes c
         JOIN user_capes uc ON c.id = uc.cape_id
         WHERE uc.user_id = $1
         ORDER BY c.uploaded_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn list_user_granted_cape_ids(pool: &PgPool, user_id: Uuid) -> Result<Vec<Uuid>> {
    let ids: Vec<Uuid> = sqlx::query_scalar("SELECT cape_id FROM user_capes WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(ids)
}

pub async fn set_user_granted_capes(pool: &PgPool, user_id: Uuid, cape_ids: &[Uuid]) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM user_capes WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    for &cape_id in cape_ids {
        sqlx::query("INSERT INTO user_capes (user_id, cape_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(user_id)
            .bind(cape_id)
            .execute(&mut *tx)
            .await?;
    }
    // Если текущий экипированный плащ игрока больше не в списке доступных — сбрасываем его
    let current_url: Option<String> = sqlx::query_scalar("SELECT cape_url FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;
    if let Some(url) = current_url {
        let still_allowed: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM capes c
                JOIN user_capes uc ON c.id = uc.cape_id
                WHERE uc.user_id = $1 AND c.url = $2
            )",
        )
        .bind(user_id)
        .bind(&url)
        .fetch_one(&mut *tx)
        .await?;
        if !still_allowed {
            sqlx::query("UPDATE users SET cape_url = NULL WHERE id = $1")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

pub async fn cape_name_exists(pool: &PgPool, name: &str) -> Result<bool> {
    Ok(
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM capes WHERE lower(name) = lower($1))")
            .bind(name)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn insert_cape(
    pool: &PgPool,
    name: &str,
    url: &str,
    file_sha1: &str,
    size: i64,
    uploaded_by: Option<Uuid>,
) -> Result<CapeRow> {
    let row = sqlx::query_as::<_, CapeDbRow>(
        "INSERT INTO capes (name, url, file_sha1, size, uploaded_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *",
    )
    .bind(name)
    .bind(url)
    .bind(file_sha1)
    .bind(size)
    .bind(uploaded_by)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn delete_cape(pool: &PgPool, id: Uuid) -> Result<bool> {
    let mut tx = pool.begin().await?;
    let url: Option<String> = sqlx::query_scalar("SELECT url FROM capes WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(url) = url else {
        tx.commit().await?;
        return Ok(false);
    };
    sqlx::query("UPDATE users SET cape_url = NULL WHERE cape_url = $1")
        .bind(&url)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM capes WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(true)
}

pub async fn get_cape_url(pool: &PgPool, id: Uuid) -> Result<Option<String>> {
    Ok(sqlx::query_scalar("SELECT url FROM capes WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn set_user_cape(pool: &PgPool, user_id: Uuid, cape_url: Option<&str>) -> Result<()> {
    sqlx::query("UPDATE users SET cape_url = $2 WHERE id = $1")
        .bind(user_id)
        .bind(cape_url)
        .execute(pool)
        .await?;
    Ok(())
}
