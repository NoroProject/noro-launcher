//! Лента разбора: что делали с делом и откуда.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CaseEventRow {
    pub id: Uuid,
    pub case_id: Uuid,
    pub at: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
    pub actor_label: String,
    /// `web`, `game` или `system` — по ленте должно быть видно, откуда действовали.
    pub source: String,
    pub kind: String,
    pub payload: serde_json::Value,
}

pub async fn add_event(
    pool: &PgPool,
    case_id: Uuid,
    actor_id: Option<Uuid>,
    actor_label: &str,
    source: &str,
    kind: &str,
    payload: serde_json::Value,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO case_events (case_id, actor_id, actor_label, source, kind, payload)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(case_id)
    .bind(actor_id)
    .bind(actor_label)
    .bind(source)
    .bind(kind)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_events(pool: &PgPool, case_id: Uuid) -> Result<Vec<CaseEventRow>> {
    Ok(sqlx::query_as::<_, CaseEventRow>(
        "SELECT * FROM case_events WHERE case_id = $1 ORDER BY at",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await?)
}
