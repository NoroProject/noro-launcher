//! Дела: разбор жалобы с лентой событий и срезом чата.
//!
//! Дело заводится на игрока и сервер, а не на жалобу: семь жалоб на одного
//! читера — один разбор. Склейка живёт в `open_case`, а не в вызывающем коде,
//! потому что дело заводят и агент, и админка, и «одно открытое» должно
//! значить для них одно и то же.

mod close;
mod events;
mod messages;
mod queue;

pub use close::*;
pub use events::*;
pub use messages::*;
pub use queue::*;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CaseRow {
    pub id: Uuid,
    /// Человеческий номер: печатается как `N-000000001`.
    pub number: i64,
    pub target_id: Uuid,
    pub game_server_id: Option<Uuid>,
    pub status: String,
    pub claimed_by: Option<Uuid>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub opened_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub verdict: Option<String>,
    pub rule_id: Option<Uuid>,
    pub rule_code: Option<String>,
    pub resolution: String,
}

pub async fn open_case(
    pool: &PgPool,
    target_id: Uuid,
    game_server_id: Option<Uuid>,
) -> Result<CaseRow> {
    if let Some(existing) = sqlx::query_as::<_, CaseRow>(
        "SELECT * FROM cases
          WHERE target_id = $1 AND game_server_id IS NOT DISTINCT FROM $2
            AND status IN ('open', 'in_review')",
    )
    .bind(target_id)
    .bind(game_server_id)
    .fetch_optional(pool)
    .await?
    {
        return Ok(existing);
    }

    Ok(sqlx::query_as::<_, CaseRow>(
        "INSERT INTO cases (target_id, game_server_id) VALUES ($1, $2) RETURNING *",
    )
    .bind(target_id)
    .bind(game_server_id)
    .fetch_one(pool)
    .await?)
}

pub async fn get_case(pool: &PgPool, id: Uuid) -> Result<Option<CaseRow>> {
    Ok(
        sqlx::query_as::<_, CaseRow>("SELECT * FROM cases WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Взять дело. Замок ставится одним UPDATE: два модератора, нажавшие кнопку
/// одновременно, не должны получить по меню разбора каждый.
pub async fn claim_case(pool: &PgPool, id: Uuid, actor_id: Uuid) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE cases SET status = 'in_review', claimed_by = $2, claimed_at = NOW()
          WHERE id = $1 AND status = 'open' AND claimed_by IS NULL",
    )
    .bind(id)
    .bind(actor_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Отпустить дело в очередь. Только тот, кто держит замок: иначе перехват
/// выглядел бы как «дело освободилось само».
pub async fn release_case(pool: &PgPool, id: Uuid, actor_id: Uuid) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE cases SET status = 'open', claimed_by = NULL, claimed_at = NULL
          WHERE id = $1 AND status = 'in_review' AND claimed_by = $2",
    )
    .bind(id)
    .bind(actor_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Открытое дело на игрока и сервер, если оно есть.
///
/// Без создания: наказание из игры должно попадать в уже идущий разбор, но
/// заводить дело на каждый мут за капс незачем.
pub async fn find_open_case(
    pool: &PgPool,
    target_id: Uuid,
    game_server_id: Option<Uuid>,
) -> Result<Option<CaseRow>> {
    Ok(sqlx::query_as::<_, CaseRow>(
        "SELECT * FROM cases
          WHERE target_id = $1 AND game_server_id IS NOT DISTINCT FROM $2
            AND status IN ('open', 'in_review')",
    )
    .bind(target_id)
    .bind(game_server_id)
    .fetch_optional(pool)
    .await?)
}

/// Дело, которое этот модератор ведёт на этом сервере.
///
/// Нужно на входе в игру: замок ставится с сайта и переживает перезапуск
/// сервера, а режим разбора у агента — нет. Без этого модератор видит «в
/// работе» в панели и «you are not reviewing any case» в чате.
pub async fn claimed_on_server(
    pool: &PgPool,
    moderator_id: Uuid,
    game_server_id: Uuid,
) -> Result<Vec<CaseRow>> {
    // Все, а не последнее: замков у одного модератора бывает несколько, и
    // агент должен знать про каждое — иначе переключиться между ними в игре
    // не выйдет, а панель будет показывать дело, которого у агента нет.
    Ok(sqlx::query_as::<_, CaseRow>(
        "SELECT * FROM cases
          WHERE claimed_by = $1 AND game_server_id = $2 AND status = 'in_review'
          ORDER BY claimed_at",
    )
    .bind(moderator_id)
    .bind(game_server_id)
    .fetch_all(pool)
    .await?)
}

/// Привязать жалобу к делу.
pub async fn attach_report(pool: &PgPool, report_id: Uuid, case_id: Uuid) -> Result<()> {
    sqlx::query("UPDATE player_reports SET case_id = $2 WHERE id = $1")
        .bind(report_id)
        .bind(case_id)
        .execute(pool)
        .await?;
    Ok(())
}
