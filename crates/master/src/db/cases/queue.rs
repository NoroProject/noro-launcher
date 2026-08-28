//! The case queue and reporter reputation.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CaseListRow {
    pub id: Uuid,
    pub number: i64,
    pub target_id: Uuid,
    pub target_name: Option<String>,
    pub game_server_id: Option<Uuid>,
    pub server_name: Option<String>,
    pub status: String,
    pub claimed_by: Option<Uuid>,
    pub claimed_by_name: Option<String>,
    pub opened_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub verdict: Option<String>,
    pub rule_code: Option<String>,
    /// Reports in the case, and how many *distinct* people filed them. Ten from
    /// one aggrieved player and ten from ten aren't worth the same.
    pub reports_count: i64,
    pub reporters_count: i64,
    pub last_report_at: Option<DateTime<Utc>>,
}

/// Kept separate from the column list so the count runs over the same joins as
/// the page. Search hits names in `users` and `game_servers`, so a `count(*)`
/// without those joins would come out different.
const FROM_SQL: &str = "
      FROM cases c
      LEFT JOIN users tu ON tu.id = c.target_id
      LEFT JOIN users mu ON mu.id = c.claimed_by
      LEFT JOIN game_servers gs ON gs.id = c.game_server_id
      LEFT JOIN (
            SELECT case_id,
                   COUNT(*)                    AS reports_count,
                   COUNT(DISTINCT reporter_id) AS reporters_count,
                   MAX(created_at)             AS last_report_at
              FROM player_reports WHERE case_id IS NOT NULL GROUP BY case_id
      ) r ON r.case_id = c.id
";

const COLUMNS_SQL: &str = "
    SELECT c.*,
           tu.mc_username AS target_name,
           gs.name        AS server_name,
           mu.mc_username AS claimed_by_name,
           COALESCE(r.reports_count, 0)   AS reports_count,
           COALESCE(r.reporters_count, 0) AS reporters_count,
           r.last_report_at
";

/// Queue search: offender name, server, whoever claimed the case, case number.
///
/// The number is compared as text — a moderator pastes "142" from someone
/// else's message and expects case 142, not a range.
///
/// `$1::text IS NULL` means "no search". The condition is always present so the
/// placeholder never disappears: Postgres derives the parameter count from the
/// highest one mentioned, and a query with $2 and $3 but no $1 won't prepare.
const SEARCH_SQL: &str = "($1::text IS NULL OR (
       tu.mc_username ILIKE $1 ESCAPE '\\'
    OR gs.name        ILIKE $1 ESCAPE '\\'
    OR mu.mc_username ILIKE $1 ESCAPE '\\'
    OR c.number::text LIKE  $1 ESCAPE '\\'
))";

/// A page of the queue plus the total under the same condition.
///
/// Open cases are ordered by distinct reporter count: five people on a cheater
/// outrank one report about swearing.
///
/// `like` is a `%…%` pattern with the wildcards already escaped — see
/// `PageQuery::like`.
pub async fn list_cases(
    pool: &PgPool,
    open_only: bool,
    like: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<CaseListRow>, i64)> {
    let where_sql = if open_only {
        format!("WHERE c.status IN ('open', 'in_review') AND {SEARCH_SQL}")
    } else {
        format!("WHERE {SEARCH_SQL}")
    };

    let order_sql = if open_only {
        "ORDER BY COALESCE(r.reporters_count, 0) DESC, c.opened_at"
    } else {
        "ORDER BY c.opened_at DESC"
    };

    let rows = sqlx::query_as::<_, CaseListRow>(&format!(
        "{COLUMNS_SQL} {FROM_SQL} {where_sql} {order_sql} LIMIT $2 OFFSET $3"
    ))
    .bind(like)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(&format!("SELECT count(*) {FROM_SQL} {where_sql}"))
        .bind(like)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

pub async fn get_case_view(pool: &PgPool, id: Uuid) -> Result<Option<CaseListRow>> {
    let sql = format!("{COLUMNS_SQL} {FROM_SQL} WHERE c.id = $1");
    Ok(sqlx::query_as::<_, CaseListRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// How many of a player's reports were upheld. Computed rather than stored —
/// there are only tens of rows per player, and a cached column would have to be
/// repaired after every edit to a case.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReporterStats {
    pub total: i64,
    pub confirmed: i64,
    pub rejected: i64,
}

pub async fn reporter_stats(pool: &PgPool, user_id: Uuid) -> Result<ReporterStats> {
    Ok(sqlx::query_as::<_, ReporterStats>(
        "SELECT COUNT(*) AS total,
                COUNT(*) FILTER (WHERE c.verdict = 'confirmed') AS confirmed,
                COUNT(*) FILTER (WHERE c.verdict = 'rejected')  AS rejected
           FROM player_reports p
           LEFT JOIN cases c ON c.id = p.case_id
          WHERE p.reporter_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

/// The same counts as `reporter_stats`, but for the player being reported.
pub async fn target_stats(pool: &PgPool, user_id: Uuid) -> Result<ReporterStats> {
    Ok(sqlx::query_as::<_, ReporterStats>(
        "SELECT COUNT(*) AS total,
                COUNT(*) FILTER (WHERE verdict = 'confirmed') AS confirmed,
                COUNT(*) FILTER (WHERE verdict = 'rejected')  AS rejected
           FROM cases WHERE target_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

/// A report inside a case, with the author's name resolved. Reputation is not
/// included here — call `reporter_stats` for it.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CaseReportRow {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub reporter_name: Option<String>,
    pub reason: String,
    pub world: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub created_at: DateTime<Utc>,
}

pub async fn case_reports(pool: &PgPool, case_id: Uuid) -> Result<Vec<CaseReportRow>> {
    Ok(sqlx::query_as::<_, CaseReportRow>(
        "SELECT p.id, p.reporter_id, u.mc_username AS reporter_name, p.reason,
                p.world, p.x, p.y, p.z, p.created_at
           FROM player_reports p
           LEFT JOIN users u ON u.id = p.reporter_id
          WHERE p.case_id = $1 ORDER BY p.created_at",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await?)
}
