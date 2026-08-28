//! Closing a case: the verdict, the reports it covers, and feedback to whoever
//! filed them.
//!
//! Skip that last step and people stop reporting within a month — nothing tells
//! them their report was ever read.

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Confirmed becomes `resolved`, anything else `rejected`. The queue is sorted
/// by status; the reason for closing lives in the verdict.
pub async fn resolve_case(
    pool: &PgPool,
    id: Uuid,
    verdict: &str,
    resolution: &str,
    rule_code: Option<&str>,
) -> Result<bool> {
    let status = if verdict == "confirmed" {
        "resolved"
    } else {
        "rejected"
    };
    let res = sqlx::query(
        "UPDATE cases
            SET status = $2, verdict = $3, resolution = $4, rule_code = $5, resolved_at = NOW()
          WHERE id = $1 AND status IN ('open', 'in_review')",
    )
    .bind(id)
    .bind(status)
    .bind(verdict)
    .bind(resolution)
    .bind(rule_code)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Close the case's reports and queue feedback for their authors.
///
/// Everyone gets the same text: a case has one verdict, and its reports are all
/// about the same incident.
pub async fn close_reports(pool: &PgPool, case_id: Uuid, resolution: &str) -> Result<u64> {
    let res = sqlx::query(
        "UPDATE player_reports
            SET status = 'resolved', resolved_at = NOW(), resolution = $2
          WHERE case_id = $1 AND status IN ('open', 'claimed')",
    )
    .bind(case_id)
    .bind(resolution)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO pending_report_feedbacks (user_id, target_username, resolution)
         SELECT DISTINCT p.reporter_id, COALESCE(u.mc_username, '-'), $2
           FROM player_reports p
           LEFT JOIN cases c ON c.id = p.case_id
           LEFT JOIN users u ON u.id = c.target_id
          WHERE p.case_id = $1",
    )
    .bind(case_id)
    .bind(resolution)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}
