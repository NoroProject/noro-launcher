//! Заморозка игроков (`player_freezes`).

use anyhow::Result;
use chrono::{DateTime, Utc};
use schema::FreezeInfo;
use sqlx::PgPool;
use uuid::Uuid;

/// Заморозить игрока.
pub async fn freeze_player(
    pool: &PgPool,
    user_id: Uuid,
    actor_id: Option<Uuid>,
    reason: &str,
) -> Result<()> {
    sqlx::query("UPDATE player_freezes SET released_at = NOW() WHERE user_id = $1 AND released_at IS NULL")
        .bind(user_id)
        .execute(pool)
        .await?;

    sqlx::query("INSERT INTO player_freezes (user_id, actor_id, reason) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(actor_id)
        .bind(reason)
        .execute(pool)
        .await?;

    Ok(())
}

/// Разморозить игрока.
pub async fn unfreeze_player(pool: &PgPool, user_id: Uuid) -> Result<bool> {
    let res = sqlx::query("UPDATE player_freezes SET released_at = NOW() WHERE user_id = $1 AND released_at IS NULL")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
struct FreezeRow {
    reason: String,
    frozen_at: DateTime<Utc>,
    frozen_by: Option<String>,
}

/// Получить действующую заморозку со сведениями (кто заморозил и причина).
pub async fn active_freeze_for_user(pool: &PgPool, user_id: Uuid) -> Result<Option<FreezeInfo>> {
    let row = sqlx::query_as::<_, FreezeRow>(
        r#"
        SELECT pf.reason, pf.created_at as frozen_at, u.mc_username as frozen_by
        FROM player_freezes pf
        LEFT JOIN users u ON u.id = pf.actor_id
        WHERE pf.user_id = $1 AND pf.released_at IS NULL
        ORDER BY pf.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| FreezeInfo {
        reason: r.reason,
        frozen_by: r.frozen_by.unwrap_or_else(|| "Console".into()),
        frozen_at: r.frozen_at,
    }))
}
