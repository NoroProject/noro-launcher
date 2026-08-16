//! Наказания: баны со сроком, предупреждения, ограничения по серверу.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PunishmentRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: String,
    pub reason: String,
    pub actor_id: Option<Uuid>,
    pub actor_label: String,
    pub server_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    /// `None` — навсегда.
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

impl PunishmentRow {
    /// Действует ли прямо сейчас.
    pub fn active(&self) -> bool {
        self.revoked_at.is_none() && self.expires_at.is_none_or(|e| e > Utc::now())
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_punishment(
    pool: &PgPool,
    user_id: Uuid,
    kind: &str,
    reason: &str,
    actor_id: Option<Uuid>,
    actor_label: &str,
    server_id: Option<Uuid>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<PunishmentRow> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "INSERT INTO punishments
           (user_id, kind, reason, actor_id, actor_label, server_id, expires_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         RETURNING *",
    )
    .bind(user_id)
    .bind(kind)
    .bind(reason)
    .bind(actor_id)
    .bind(actor_label)
    .bind(server_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await?)
}

/// Вся история, включая снятое: снятый бан — это тоже факт, который нужен при
/// разборе следующего случая.
pub async fn list_punishments(pool: &PgPool, user_id: Uuid) -> Result<Vec<PunishmentRow>> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "SELECT * FROM punishments WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// Действующие наказания.
pub async fn active_punishments(pool: &PgPool, user_id: Uuid) -> Result<Vec<PunishmentRow>> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "SELECT * FROM punishments
         WHERE user_id = $1 AND revoked_at IS NULL
           AND (expires_at IS NULL OR expires_at > NOW())
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn revoke_punishment(pool: &PgPool, id: Uuid, by: Option<Uuid>) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE punishments SET revoked_at = NOW(), revoked_by = $2
         WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(by)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Отметить предупреждение прочитанным — игрок подтвердил перед входом.
pub async fn acknowledge_punishment(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE punishments SET acknowledged_at = NOW()
         WHERE id = $1 AND user_id = $2 AND acknowledged_at IS NULL",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Синхронизировать денормализованный `users.banned`.
///
/// Флаг остаётся как кэш: по нему ходят Yggdrasil и WS-вход, и заменить их
/// запросом в `punishments` на каждом обращении к игре — лишняя цена.
pub async fn refresh_ban_flag(pool: &PgPool, user_id: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "UPDATE users u
         SET banned = EXISTS (
                 SELECT 1 FROM punishments p
                 WHERE p.user_id = u.id AND p.kind = 'ban' AND p.revoked_at IS NULL
                   AND (p.expires_at IS NULL OR p.expires_at > NOW())
             ),
             ban_reason = (
                 SELECT p.reason FROM punishments p
                 WHERE p.user_id = u.id AND p.kind = 'ban' AND p.revoked_at IS NULL
                   AND (p.expires_at IS NULL OR p.expires_at > NOW())
                 ORDER BY p.created_at DESC LIMIT 1
             )
         WHERE u.id = $1
         RETURNING banned",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

/// Снять истёкшие баны с флага. Без этого временный бан истекает в таблице, но
/// игрок остаётся заблокированным до следующей правки его карточки.
pub async fn expire_punishments(pool: &PgPool) -> Result<u64> {
    Ok(sqlx::query(
        "UPDATE users u SET banned = FALSE, ban_reason = NULL
         WHERE u.banned
           AND NOT EXISTS (
               SELECT 1 FROM punishments p
               WHERE p.user_id = u.id AND p.kind = 'ban' AND p.revoked_at IS NULL
                 AND (p.expires_at IS NULL OR p.expires_at > NOW())
           )",
    )
    .execute(pool)
    .await?
    .rows_affected())
}
