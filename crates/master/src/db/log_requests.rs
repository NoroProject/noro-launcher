//! Запросы логов у игрока.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// Сколько живёт запрос в очереди. До 24 часов, чтобы при открытии лаунчера
/// игрок получал отложенные запросы из очереди.
pub const REQUEST_TTL_MINS: i64 = 1440;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LogRequestRow {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_label: String,
    pub target_id: Uuid,
    pub reason: String,
    pub server_id: Option<Uuid>,
    pub forced: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub answered_at: Option<DateTime<Utc>>,
    pub bundle_id: Option<Uuid>,
}

pub async fn create_log_request(
    pool: &PgPool,
    actor_id: Option<Uuid>,
    actor_label: &str,
    target_id: Uuid,
    reason: &str,
    server_id: Option<Uuid>,
    forced: bool,
) -> Result<LogRequestRow> {
    Ok(sqlx::query_as::<_, LogRequestRow>(
        "INSERT INTO log_requests
           (actor_id, actor_label, target_id, reason, server_id, forced, expires_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         RETURNING *",
    )
    .bind(actor_id)
    .bind(actor_label)
    .bind(target_id)
    .bind(reason)
    .bind(server_id)
    .bind(forced)
    .bind(Utc::now() + Duration::minutes(REQUEST_TTL_MINS))
    .fetch_one(pool)
    .await?)
}

pub async fn get_log_request(pool: &PgPool, id: Uuid) -> Result<Option<LogRequestRow>> {
    Ok(
        sqlx::query_as::<_, LogRequestRow>("SELECT * FROM log_requests WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Ответ игрока. Только свой запрос и только пока он жив.
pub async fn answer_log_request(
    pool: &PgPool,
    id: Uuid,
    target_id: Uuid,
    accepted: bool,
) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE log_requests
         SET status = $3, answered_at = NOW()
         WHERE id = $1 AND target_id = $2 AND status = 'pending' AND expires_at > NOW()",
    )
    .bind(id)
    .bind(target_id)
    .bind(if accepted { "accepted" } else { "declined" })
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// Привязать доставленный бандл.
pub async fn deliver_log_request(pool: &PgPool, id: Uuid, bundle_id: Uuid) -> Result<()> {
    sqlx::query("UPDATE log_requests SET status = 'delivered', bundle_id = $2 WHERE id = $1")
        .bind(id)
        .bind(bundle_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Можно ли по этому запросу принять бандл прямо сейчас.
///
/// Принудительный не спрашивает согласия — но и он протухает: запрос,
/// провисевший неделю, это уже не тот разбор, ради которого его заводили.
pub async fn request_open_for(pool: &PgPool, id: Uuid, target_id: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM log_requests
             WHERE id = $1 AND target_id = $2 AND expires_at > NOW()
               AND (status = 'accepted' OR (forced AND status = 'pending'))
         )",
    )
    .bind(id)
    .bind(target_id)
    .fetch_one(pool)
    .await?)
}

pub async fn list_log_requests(
    pool: &PgPool,
    target_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<LogRequestRow>> {
    Ok(sqlx::query_as::<_, LogRequestRow>(
        "SELECT * FROM log_requests
         WHERE ($1::uuid IS NULL OR target_id = $1)
         ORDER BY created_at DESC
         LIMIT $2",
    )
    .bind(target_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?)
}

/// Пометить протухшие. Иначе «ожидает ответа» висит вечно и врёт.
pub async fn expire_log_requests(pool: &PgPool) -> Result<u64> {
    Ok(sqlx::query(
        "UPDATE log_requests SET status = 'expired'
         WHERE status = 'pending' AND expires_at < NOW()",
    )
    .execute(pool)
    .await?
    .rows_affected())
}

/// Выборка накопившихся невыполненных запросов для пользователя.
pub async fn list_pending_for_user(pool: &PgPool, target_id: Uuid) -> Result<Vec<LogRequestRow>> {
    expire_log_requests(pool).await?;
    Ok(sqlx::query_as::<_, LogRequestRow>(
        "SELECT * FROM log_requests
         WHERE target_id = $1 AND status = 'pending' AND expires_at > NOW()
         ORDER BY created_at ASC",
    )
    .bind(target_id)
    .fetch_all(pool)
    .await?)
}

/// Отменить отложенный запрос логов.
pub async fn cancel_log_request(pool: &PgPool, id: Uuid) -> Result<Option<LogRequestRow>> {
    Ok(sqlx::query_as::<_, LogRequestRow>(
        "UPDATE log_requests
         SET status = 'cancelled', answered_at = NOW()
         WHERE id = $1 AND status = 'pending'
         RETURNING *",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}
