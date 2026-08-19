//! Игровые сессии и годовой heatmap активности.

use anyhow::Result;
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ActivityDayRow {
    pub day: NaiveDate,
    pub minutes: i32,
}

/// Открыть сессию при входе игрока на сервер.
pub async fn start_session(pool: &PgPool, mc_uuid: Uuid, game_server_id: Uuid) -> Result<()> {
    let user_id: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE mc_uuid = $1")
            .bind(mc_uuid)
            .fetch_optional(pool)
            .await?;
    if let Some(user_id) = user_id {
        // Закрываем висячие предыдущие сессии на этом же сервере
        sqlx::query(
            "UPDATE player_sessions SET ended_at = NOW(), end_reason = 'rejoined'
             WHERE user_id = $1 AND game_server_id = $2 AND ended_at IS NULL",
        )
        .bind(user_id)
        .bind(game_server_id)
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT INTO player_sessions (user_id, game_server_id, started_at)
             VALUES ($1, $2, NOW())",
        )
        .bind(user_id)
        .bind(game_server_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Закрыть сессию при выходе игрока.
pub async fn end_session(pool: &PgPool, mc_uuid: Uuid, game_server_id: Uuid, reason: &str) -> Result<()> {
    let user_id: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE mc_uuid = $1")
            .bind(mc_uuid)
            .fetch_optional(pool)
            .await?;
    if let Some(user_id) = user_id {
        let rows = sqlx::query(
            "UPDATE player_sessions
             SET ended_at = NOW(), end_reason = $3
             WHERE user_id = $1 AND game_server_id = $2 AND ended_at IS NULL
             RETURNING id, started_at",
        )
        .bind(user_id)
        .bind(game_server_id)
        .bind(reason)
        .fetch_all(pool)
        .await?;

        for row in rows {
            let started_at: chrono::DateTime<chrono::Utc> = sqlx::Row::get(&row, "started_at");
            let mins = (chrono::Utc::now() - started_at).num_minutes().max(1) as i32;
            sqlx::query(
                "INSERT INTO player_activity_days (user_id, day, minutes)
                 VALUES ($1, CURRENT_DATE, $2)
                 ON CONFLICT (user_id, day) DO UPDATE
                 SET minutes = player_activity_days.minutes + EXCLUDED.minutes",
            )
            .bind(user_id)
            .bind(mins)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

/// Запросить сетку активности игрока за последние 365 дней.
pub async fn get_activity_heatmap(pool: &PgPool, user_id: Uuid) -> Result<Vec<ActivityDayRow>> {
    Ok(sqlx::query_as::<_, ActivityDayRow>(
        "SELECT day, SUM(minutes)::INT AS minutes FROM (
             SELECT day, minutes FROM player_activity_days WHERE user_id = $1 AND day >= CURRENT_DATE - INTERVAL '365 days'
             UNION ALL
             SELECT CURRENT_DATE AS day, GREATEST(1, EXTRACT(EPOCH FROM (NOW() - started_at)) / 60)::INT AS minutes
             FROM player_sessions
             WHERE user_id = $1 AND ended_at IS NULL
         ) s
         GROUP BY day
         ORDER BY day ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}
