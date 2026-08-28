//! The chat excerpt attached to a case.
//!
//! There is no general chat log. The agent keeps recent messages in memory and
//! hands over a window only when something happens — a report or a punishment.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CaseMessageRow {
    pub id: Uuid,
    pub at: DateTime<Utc>,
    pub sender_name: String,
    /// `public`, `local`, `private` or `command`.
    pub channel: String,
    pub content: String,
}

/// A message from the agent's buffer, not yet attached to a case.
#[derive(Debug, Clone, Deserialize)]
pub struct IncomingMessage {
    pub at: DateTime<Utc>,
    #[serde(default)]
    pub sender: Option<Uuid>,
    pub sender_name: String,
    pub channel: String,
    pub content: String,
}

pub async fn list_messages(pool: &PgPool, case_id: Uuid) -> Result<Vec<CaseMessageRow>> {
    Ok(sqlx::query_as::<_, CaseMessageRow>(
        "SELECT id, at, sender_name, channel, content FROM case_messages
          WHERE case_id = $1 ORDER BY at",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await?)
}

/// The excerpt line nearest to a given time. Client and server clocks drift, so
/// this searches a window and takes the closest hit rather than matching exactly.
pub async fn message_near(
    pool: &PgPool,
    case_id: Uuid,
    sender_name: &str,
    at: DateTime<Utc>,
    window_secs: i64,
) -> Result<Option<CaseMessageRow>> {
    Ok(sqlx::query_as::<_, CaseMessageRow>(
        "SELECT id, at, sender_name, channel, content FROM case_messages
          WHERE case_id = $1 AND sender_name = $2
            AND at BETWEEN $3 - make_interval(secs => $4) AND $3 + make_interval(secs => $4)
          ORDER BY abs(extract(epoch FROM (at - $3))) LIMIT 1",
    )
    .bind(case_id)
    .bind(sender_name)
    .bind(at)
    .bind(window_secs as f64)
    .fetch_optional(pool)
    .await?)
}

/// Save an excerpt. Duplicates are dropped on (time, sender): the agent sends a
/// window on both reports and punishments, and those windows overlap.
pub async fn save_messages(
    pool: &PgPool,
    case_id: Uuid,
    messages: &[IncomingMessage],
) -> Result<u64> {
    let mut saved = 0;
    for m in messages {
        let res = sqlx::query(
            "INSERT INTO case_messages (case_id, at, sender_id, sender_name, channel, content)
             SELECT $1, $2, $3, $4, $5, $6
              WHERE NOT EXISTS (
                    SELECT 1 FROM case_messages
                     WHERE case_id = $1 AND at = $2 AND sender_name = $4)",
        )
        .bind(case_id)
        .bind(m.at)
        .bind(m.sender)
        .bind(&m.sender_name)
        .bind(&m.channel)
        .bind(&m.content)
        .execute(pool)
        .await?;
        saved += res.rows_affected();
    }
    Ok(saved)
}
