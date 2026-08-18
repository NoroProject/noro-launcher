//! Правки журнала наказаний и денормализованного флага бана.

use super::{NewPunishment, PunishmentRow};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_punishment(pool: &PgPool, new: NewPunishment<'_>) -> Result<PunishmentRow> {
    Ok(sqlx::query_as::<_, PunishmentRow>(
        "INSERT INTO punishments
           (user_id, kind, reason, actor_id, actor_label, server_id, expires_at,
            rule_id, rule_code)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         RETURNING *",
    )
    .bind(new.user_id)
    .bind(new.kind)
    .bind(new.reason)
    .bind(new.actor_id)
    .bind(new.actor_label)
    .bind(new.server_id)
    .bind(new.expires_at)
    .bind(new.rule_id)
    .bind(new.rule_code)
    .fetch_one(pool)
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
