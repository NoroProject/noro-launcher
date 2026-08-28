//! Cases: a report under review, with an event feed and a chat excerpt.
//!
//! A case belongs to a player and a server, not to a report — seven reports
//! about one cheater are one review. The merging happens inside `open_case`
//! rather than at the call sites, because both the agent and the admin UI open
//! cases and they have to agree on what "already open" means.

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
    /// Human-facing number, printed as `N-000000001`.
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

/// Claim a case. The lock is taken in a single UPDATE so that two moderators
/// pressing the button at once don't both end up reviewing it.
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

/// Put a case back in the queue. Only the holder can do this — otherwise a
/// takeover would look to them like the case released itself.
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

/// Look up an open case without creating one. A punishment issued in-game
/// should land in a review that's already running, but every mute for caps
/// doesn't need a case of its own.
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

/// Cases this moderator holds on this server.
///
/// Read when they join the game: the lock is taken on the site and survives a
/// server restart, but the agent's review mode doesn't. Without this they'd see
/// "in review" in the panel and "you are not reviewing any case" in chat.
pub async fn claimed_on_server(
    pool: &PgPool,
    moderator_id: Uuid,
    game_server_id: Uuid,
) -> Result<Vec<CaseRow>> {
    // All of them, not just the latest — one moderator can hold several locks,
    // and the agent needs every one to let them switch between cases in game.
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

pub async fn attach_report(pool: &PgPool, report_id: Uuid, case_id: Uuid) -> Result<()> {
    sqlx::query("UPDATE player_reports SET case_id = $2 WHERE id = $1")
        .bind(report_id)
        .bind(case_id)
        .execute(pool)
        .await?;
    Ok(())
}
