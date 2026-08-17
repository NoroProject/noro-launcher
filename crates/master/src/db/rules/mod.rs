//! Свод правил: разделы и правила.
//!
//! Запросы всегда работают со «сводом» целиком — общим или общим плюс
//! дополнением одного сервера. Отдельно серверные правила не отдаются: игрок,
//! которому показали только дополнение, решил бы, что остального не существует.

mod edit;
mod sanctions;
mod translations;

pub use edit::*;
pub use sanctions::*;
pub use translations::*;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RuleCategoryRow {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RuleRow {
    pub id: Uuid,
    pub category_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
    pub code: String,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Разделы свода: общие плюс, если задан сервер, его собственные.
pub async fn list_rule_categories(
    pool: &PgPool,
    server_id: Option<Uuid>,
) -> Result<Vec<RuleCategoryRow>> {
    let rows = sqlx::query_as::<_, RuleCategoryRow>(
        "SELECT * FROM rule_categories
         WHERE server_id IS NULL OR server_id = $1
         ORDER BY sort_order, code, name",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Все разделы всех сводов — только для админки.
pub async fn list_all_rule_categories(pool: &PgPool) -> Result<Vec<RuleCategoryRow>> {
    let rows = sqlx::query_as::<_, RuleCategoryRow>(
        "SELECT * FROM rule_categories ORDER BY sort_order, code, name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Правила свода: общие плюс, если задан сервер, его собственные.
pub async fn list_rules(pool: &PgPool, server_id: Option<Uuid>) -> Result<Vec<RuleRow>> {
    let rows = sqlx::query_as::<_, RuleRow>(
        "SELECT * FROM rules
         WHERE server_id IS NULL OR server_id = $1
         ORDER BY sort_order, code",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_all_rules(pool: &PgPool) -> Result<Vec<RuleRow>> {
    let rows = sqlx::query_as::<_, RuleRow>("SELECT * FROM rules ORDER BY sort_order, code")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// Серверы, у которых есть собственные правила: только они и имеют смысл в
/// переключателе сводов — остальные показали бы ту же общую страницу.
pub async fn list_rule_scopes(pool: &PgPool) -> Result<Vec<(Uuid, String)>> {
    let rows = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT s.id, s.name FROM servers s
         WHERE EXISTS (SELECT 1 FROM rules r WHERE r.server_id = s.id)
            OR EXISTS (SELECT 1 FROM rule_categories c WHERE c.server_id = s.id)
         ORDER BY s.sort_order, s.name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn rule_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RuleRow>> {
    let row = sqlx::query_as::<_, RuleRow>("SELECT * FROM rules WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// Правило по коду в пределах свода — по нему ссылаются из игры и из тикетов.
pub async fn rule_by_code(
    pool: &PgPool,
    code: &str,
    server_id: Option<Uuid>,
) -> Result<Option<RuleRow>> {
    let row = sqlx::query_as::<_, RuleRow>(
        "SELECT * FROM rules
         WHERE lower(code) = lower($1) AND (server_id IS NULL OR server_id = $2)
         -- Дополнение сервера перекрывает общее правило с тем же кодом.
         ORDER BY server_id NULLS LAST
         LIMIT 1",
    )
    .bind(code)
    .bind(server_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
