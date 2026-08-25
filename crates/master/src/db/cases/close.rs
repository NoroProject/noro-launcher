//! Закрытие дела: вердикт, закрытые жалобы и обратная связь их авторам.
//!
//! Без последнего шага жалобы перестают писать через месяц: человек не видит,
//! что его обращение вообще прочитали.

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Закрыть дело. Подтвердилось — `resolved`, остальное — `rejected`: очередь
/// разбирают по статусу, а причина закрытия живёт в вердикте.
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

/// Закрыть жалобы дела и разложить авторам обратную связь.
///
/// Текст один на всех: жалобы в деле про одно и то же событие, и рассылать
/// разные ответы было бы неоткуда — вердикт у дела единственный.
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
