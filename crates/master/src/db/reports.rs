//! Журнал жалоб на игроков (`player_reports`) и обратная связь репортерам.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReportRow {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub target_id: Uuid,
    pub game_server_id: Uuid,
    pub reason: String,
    pub world: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub status: String,
    pub claimed_by: Option<Uuid>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
    pub punishment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportFeedback {
    pub target_username: String,
    pub resolution: String,
}

/// Новая жалоба. Место — четыре поля, которые едут вместе: без мира координаты
/// ничего не значат, а без координат мир бесполезен.
pub struct NewReport<'a> {
    pub reporter_id: Uuid,
    pub target_id: Uuid,
    pub game_server_id: Uuid,
    pub reason: &'a str,
    pub world: Option<&'a str>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}

pub async fn create_report(pool: &PgPool, r: NewReport<'_>) -> Result<Uuid> {
    let NewReport {
        reporter_id,
        target_id,
        game_server_id,
        reason,
        world,
        x,
        y,
        z,
    } = r;
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO player_reports (reporter_id, target_id, game_server_id, reason, world, x, y, z)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(reporter_id)
    .bind(target_id)
    .bind(game_server_id)
    .bind(reason)
    .bind(world)
    .bind(x)
    .bind(y)
    .bind(z)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Жалобы со счётчиком по тому же условию.
///
/// Зашитый `LIMIT 100` без счётчика прятал остальные и выглядел как «жалоб
/// больше нет»; открытые при этом не ограничивались ничем.
pub async fn list_reports(
    pool: &PgPool,
    open_only: bool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ReportRow>, i64)> {
    let where_sql = if open_only {
        "WHERE status IN ('open', 'claimed')"
    } else {
        ""
    };

    let rows = sqlx::query_as::<_, ReportRow>(&format!(
        "SELECT * FROM player_reports {where_sql} ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    ))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 =
        sqlx::query_scalar(&format!("SELECT count(*) FROM player_reports {where_sql}"))
            .fetch_one(pool)
            .await?;

    Ok((rows, total))
}

pub async fn claim_report(pool: &PgPool, report_id: Uuid, actor_id: Uuid) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE player_reports SET status = 'claimed', claimed_by = $2, claimed_at = NOW()
         WHERE id = $1 AND status = 'open'",
    )
    .bind(report_id)
    .bind(actor_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn resolve_report(
    pool: &PgPool,
    report_id: Uuid,
    _actor_id: Uuid,
    resolution: &str,
    punishment_id: Option<Uuid>,
) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE player_reports
         SET status = 'resolved', resolved_at = NOW(), resolution = $2, punishment_id = $3
         WHERE id = $1 AND status IN ('open', 'claimed')",
    )
    .bind(report_id)
    .bind(resolution)
    .bind(punishment_id)
    .execute(pool)
    .await?;

    if res.rows_affected() > 0 {
        let _ = sqlx::query(
            "INSERT INTO pending_report_feedbacks (user_id, target_username, resolution)
             SELECT r.reporter_id, COALESCE(u.mc_username, 'игрока'), $2
             FROM player_reports r
             LEFT JOIN users u ON u.id = r.target_id
             WHERE r.id = $1",
        )
        .bind(report_id)
        .bind(resolution)
        .execute(pool)
        .await;
        return Ok(true);
    }
    Ok(false)
}

#[derive(sqlx::FromRow)]
struct FeedbackRow {
    target_username: String,
    resolution: String,
}

pub async fn pop_pending_report_feedbacks(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ReportFeedback>> {
    let rows = sqlx::query_as::<_, FeedbackRow>(
        "DELETE FROM pending_report_feedbacks WHERE user_id = $1
         RETURNING target_username, resolution",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ReportFeedback {
            target_username: r.target_username,
            resolution: r.resolution,
        })
        .collect())
}
