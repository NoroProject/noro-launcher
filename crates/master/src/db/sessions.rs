//! Активные сессии пользователя.
//!
//! Раньше отозвать скомпрометированный токен было нечем: строка жила до
//! истечения срока, и «выйти на всех устройствах» означало ждать месяц.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SessionRow {
    pub id: Uuid,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// Кем сессия открыта, если это impersonation.
    pub impersonated_by: Option<Uuid>,
    /// Та ли это сессия, из которой пришёл запрос.
    #[sqlx(default)]
    pub current: bool,
}

pub async fn list_sessions(
    pool: &PgPool,
    user_id: Uuid,
    current_token: Option<Uuid>,
) -> Result<Vec<SessionRow>> {
    Ok(sqlx::query_as::<_, SessionRow>(
        "SELECT id, scope, created_at, expires_at, impersonated_by,
                (access_token = $2) AS current
         FROM oauth_sessions
         WHERE user_id = $1 AND expires_at > NOW()
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(current_token)
    .fetch_all(pool)
    .await?)
}

/// Завершить одну сессию. Ограничение по владельцу обязательно: иначе id чужой
/// сессии становится кнопкой «выкинуть кого угодно».
pub async fn revoke_session(pool: &PgPool, id: Uuid, user_id: Option<Uuid>) -> Result<bool> {
    let res = sqlx::query(
        "DELETE FROM oauth_sessions WHERE id = $1 AND ($2::uuid IS NULL OR user_id = $2)",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Завершить все сессии игрока, кроме, возможно, текущей.
pub async fn revoke_all_sessions(
    pool: &PgPool,
    user_id: Uuid,
    except_token: Option<Uuid>,
) -> Result<u64> {
    Ok(sqlx::query(
        "DELETE FROM oauth_sessions
         WHERE user_id = $1 AND ($2::uuid IS NULL OR access_token <> $2)",
    )
    .bind(user_id)
    .bind(except_token)
    .execute(pool)
    .await?
    .rows_affected())
}
