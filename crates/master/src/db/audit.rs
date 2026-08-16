//! Запись и чтение журнала админских действий.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditRow {
    pub id: i64,
    pub at: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
    pub actor_label: String,
    pub action: String,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub details: Value,
    pub ip: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_audit(
    pool: &PgPool,
    actor_id: Option<Uuid>,
    actor_label: &str,
    action: &str,
    target_kind: Option<&str>,
    target_id: Option<&str>,
    details: &Value,
    ip: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO audit_log
           (actor_id, actor_label, action, target_kind, target_id, details, ip)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(actor_id)
    .bind(actor_label)
    .bind(action)
    .bind(target_kind)
    .bind(target_id)
    .bind(details)
    .bind(ip)
    .execute(pool)
    .await?;
    Ok(())
}

/// Фильтр журнала. Пустые поля не сужают выборку.
#[derive(Debug, Default)]
pub struct AuditFilter {
    pub actor_id: Option<Uuid>,
    pub action: Option<String>,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub before_id: Option<i64>,
}

/// Страница журнала, новые сверху. Пагинация по id, а не по offset: журнал
/// пополняется во время просмотра, и offset начал бы показывать одно и то же.
pub async fn list_audit(pool: &PgPool, f: &AuditFilter, limit: i64) -> Result<Vec<AuditRow>> {
    let rows = sqlx::query_as::<_, AuditRow>(
        "SELECT * FROM audit_log
         WHERE ($1::uuid IS NULL OR actor_id = $1)
           AND ($2::text IS NULL OR action LIKE $2 || '%')
           AND ($3::text IS NULL OR target_kind = $3)
           AND ($4::text IS NULL OR target_id = $4)
           AND ($5::bigint IS NULL OR id < $5)
         ORDER BY id DESC
         LIMIT $6",
    )
    .bind(f.actor_id)
    .bind(&f.action)
    .bind(&f.target_kind)
    .bind(&f.target_id)
    .bind(f.before_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Какие события реально встречаются в журнале.
pub async fn distinct_audit_actions(pool: &PgPool) -> Result<Vec<String>> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT DISTINCT action FROM audit_log ORDER BY action")
            .fetch_all(pool)
            .await?,
    )
}
