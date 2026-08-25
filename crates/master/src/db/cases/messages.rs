//! Срез чата, приложенный к делу.
//!
//! Общий чат-лог не ведётся: агент держит последние сообщения в памяти и
//! отдаёт окно, только когда появился повод — жалоба или наказание.

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
    /// `public`, `local`, `private` или `command`.
    pub channel: String,
    pub content: String,
}

/// Сообщение из буфера агента, ещё не привязанное к делу.
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

/// Строка среза рядом с указанным временем. Часы клиента и сервера расходятся,
/// поэтому ищем в окне, а не по точному совпадению; ближайшая и есть та самая.
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

/// Сохранить срез. Повтор того же окна отбрасывается по (время, отправитель):
/// агент шлёт срез и при жалобе, и при наказании, и они перекрываются.
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
