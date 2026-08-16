//! Гранты impersonation и окна step-up.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// Сколько живёт одноразовый грант. Ровно столько, чтобы лаунчер успел его
/// забрать, — не дольше.
pub const GRANT_TTL_SECS: i64 = 60;

/// Сколько живёт окно step-up. Внутри него повторно подтверждаться не нужно:
/// иначе десять recovery-кодов сгорели бы за десять входов там, где passkey
/// недоступен.
pub const STEP_UP_WINDOW_MINS: i64 = 15;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GrantRow {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub target_id: Uuid,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub accepted: Option<bool>,
}

impl GrantRow {
    /// Состояние для поллинга из веба.
    pub fn status(&self) -> &'static str {
        if self.consumed_at.is_some() {
            "consumed"
        } else if self.revoked_at.is_some() || self.accepted == Some(false) {
            "declined"
        } else if self.expires_at < Utc::now() {
            "expired"
        } else if self.accepted == Some(true) {
            "accepted"
        } else {
            "pending"
        }
    }
}

pub async fn create_grant(
    pool: &PgPool,
    actor_id: Uuid,
    target_id: Uuid,
    reason: &str,
) -> Result<GrantRow> {
    Ok(sqlx::query_as::<_, GrantRow>(
        "INSERT INTO impersonation_grants (actor_id, target_id, reason, expires_at)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(actor_id)
    .bind(target_id)
    .bind(reason)
    .bind(Utc::now() + Duration::seconds(GRANT_TTL_SECS))
    .fetch_one(pool)
    .await?)
}

pub async fn get_grant(pool: &PgPool, id: Uuid) -> Result<Option<GrantRow>> {
    Ok(
        sqlx::query_as::<_, GrantRow>("SELECT * FROM impersonation_grants WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Записать ответ админа из лаунчера.
pub async fn set_grant_accepted(pool: &PgPool, id: Uuid, accepted: bool) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE impersonation_grants SET accepted = $2
         WHERE id = $1 AND accepted IS NULL AND consumed_at IS NULL AND expires_at > NOW()",
    )
    .bind(id)
    .bind(accepted)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// Сжечь грант. Возвращает его, только если он ещё был жив и подтверждён —
/// повторный обмен ничего не даёт.
pub async fn consume_grant(pool: &PgPool, id: Uuid, actor_id: Uuid) -> Result<Option<GrantRow>> {
    Ok(sqlx::query_as::<_, GrantRow>(
        "UPDATE impersonation_grants SET consumed_at = NOW()
         WHERE id = $1 AND actor_id = $2
           AND consumed_at IS NULL AND revoked_at IS NULL
           AND accepted = TRUE AND expires_at > NOW()
         RETURNING *",
    )
    .bind(id)
    .bind(actor_id)
    .fetch_optional(pool)
    .await?)
}

/// Создать сессию от имени игрока, помеченную настоящим автором.
pub async fn create_impersonated_session(
    pool: &PgPool,
    target_id: Uuid,
    actor_id: Uuid,
    ttl: Duration,
) -> Result<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO oauth_sessions (user_id, scope, expires_at, impersonated_by)
         VALUES ($1, 'launcher', $2, $3)
         RETURNING access_token",
    )
    .bind(target_id)
    .bind(Utc::now() + ttl)
    .bind(actor_id)
    .fetch_one(pool)
    .await?)
}

/// Кем открыта сессия, если это impersonation.
pub async fn session_impersonated_by(pool: &PgPool, access_token: Uuid) -> Result<Option<Uuid>> {
    Ok(sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT impersonated_by FROM oauth_sessions WHERE access_token = $1",
    )
    .bind(access_token)
    .fetch_optional(pool)
    .await?
    .flatten())
}

/// Открыть окно step-up.
pub async fn open_step_up(pool: &PgPool, user_id: Uuid, method: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO step_up_windows (user_id, method, expires_at)
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE
           SET method = $2, confirmed_at = NOW(), expires_at = $3",
    )
    .bind(user_id)
    .bind(method)
    .bind(Utc::now() + Duration::minutes(STEP_UP_WINDOW_MINS))
    .execute(pool)
    .await?;
    Ok(())
}

/// Открыто ли окно step-up прямо сейчас.
pub async fn step_up_active(pool: &PgPool, user_id: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM step_up_windows WHERE user_id = $1 AND expires_at > NOW())",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}
