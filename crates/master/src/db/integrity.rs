//! Флаги целостности от лаунчеров.

use anyhow::Result;
use chrono::{DateTime, Utc};
use schema::IntegrityReport;

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IntegrityFlagRow {
    pub id: i64,
    pub at: DateTime<Utc>,
    pub user_id: Uuid,
    pub server_id: Option<Uuid>,
    pub build_id: Option<Uuid>,
    pub build_version: String,
    pub launcher_version: String,
    pub kind: String,
    pub subject: String,
    pub detail: Option<String>,
    pub repaired: bool,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
}

/// Снимок последней сверки: с чем игрок зашёл.
///
/// Отдельно от флагов: флаги пишутся только при расхождении, а версия сборки
/// нужна журналу запусков и у тех, у кого всё сошлось.
pub async fn save_integrity_snapshot(
    pool: &PgPool,
    user_id: Uuid,
    report: &IntegrityReport,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO integrity_snapshots
           (user_id, server_id, build_version, launcher_version, enabled_optional, ok, at)
         VALUES ($1,$2,$3,$4,$5,$6,NOW())
         ON CONFLICT (user_id, server_id) DO UPDATE
           SET build_version = $3, launcher_version = $4,
               enabled_optional = $5, ok = $6, at = NOW()",
    )
    .bind(user_id)
    .bind(report.server_id)
    .bind(&report.build_version)
    .bind(&report.launcher_version)
    .bind(serde_json::to_value(&report.enabled_optional)?)
    .bind(report.findings.is_empty())
    .execute(pool)
    .await?;
    Ok(())
}

/// Снимок для журнала запусков: версия сборки, версия лаунчера, моды, статус.
#[allow(clippy::type_complexity)]
pub async fn latest_integrity_snapshot(
    pool: &PgPool,
    user_id: Uuid,
    server_id: Uuid,
) -> Result<Option<(String, String, serde_json::Value, Option<bool>)>> {
    let row = sqlx::query_as::<_, (String, String, serde_json::Value, bool)>(
        "SELECT build_version, launcher_version, enabled_optional, ok
         FROM integrity_snapshots WHERE user_id = $1 AND server_id = $2",
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(b, l, o, ok)| (b, l, o, Some(ok))))
}

/// Сохранить находки отчёта. Чистый отчёт строк не создаёт — иначе таблица
/// растёт на каждый запуск каждого игрока ради «всё в порядке».
pub async fn save_integrity_report(
    pool: &PgPool,
    user_id: Uuid,
    report: &IntegrityReport,
) -> Result<u64> {
    let mut saved = 0;
    for f in &report.findings {
        sqlx::query(
            "INSERT INTO integrity_flags
               (user_id, server_id, build_id, build_version, launcher_version,
                kind, subject, detail, repaired)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(user_id)
        .bind(report.server_id)
        .bind(report.build_id)
        .bind(&report.build_version)
        .bind(&report.launcher_version)
        .bind(f.kind.as_str())
        .bind(&f.subject)
        .bind(&f.detail)
        .bind(f.repaired)
        .execute(pool)
        .await?;
        saved += 1;
    }
    Ok(saved)
}

/// Флаги: все либо только неразобранные.
pub async fn list_integrity_flags(
    pool: &PgPool,
    user_id: Option<Uuid>,
    only_open: bool,
    limit: i64,
) -> Result<Vec<IntegrityFlagRow>> {
    let rows = sqlx::query_as::<_, IntegrityFlagRow>(
        "SELECT * FROM integrity_flags
         WHERE ($1::uuid IS NULL OR user_id = $1)
           AND ($2 = FALSE OR reviewed_at IS NULL)
         ORDER BY id DESC
         LIMIT $3",
    )
    .bind(user_id)
    .bind(only_open)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Пометить флаг разобранным. Запись остаётся: смысл журнала в том, чтобы
/// через полгода было видно, что это уже третий такой случай.
pub async fn review_integrity_flag(pool: &PgPool, id: i64, by: Option<Uuid>) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE integrity_flags SET reviewed_at = NOW(), reviewed_by = $2
         WHERE id = $1 AND reviewed_at IS NULL",
    )
    .bind(id)
    .bind(by)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Сколько неразобранных флагов у игрока — для бейджа на карточке.
pub async fn open_integrity_flag_count(pool: &PgPool, user_id: Uuid) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM integrity_flags WHERE user_id = $1 AND reviewed_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

/// Сохранить последнюю диагностику. История не хранится: важно текущее
/// состояние, а не то, сколько места было на диске в прошлый вторник.
pub async fn save_diagnostics(
    pool: &PgPool,
    user_id: Uuid,
    report: &serde_json::Value,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO launcher_diagnostics (user_id, report)
         VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE SET report = $2, at = NOW()",
    )
    .bind(user_id)
    .bind(report)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn latest_diagnostics(pool: &PgPool, user_id: Uuid) -> Result<Option<serde_json::Value>> {
    Ok(sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT jsonb_set(report, '{at}', to_jsonb(at)) FROM launcher_diagnostics WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}
