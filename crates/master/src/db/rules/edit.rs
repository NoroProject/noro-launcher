//! Правка свода: создание, изменение, удаление и порядок.

use super::{RuleCategoryRow, RuleRow};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Поля, которые задаёт админ. Одна структура на создание и правку: наборы
/// совпадают, а разъехавшись, они каждый раз забывали новое поле в одной из
/// половин.
#[derive(Debug, Clone)]
pub struct CategoryInput<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub parent_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct RuleInput<'a> {
    pub code: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    /// Формулировка для наказания по этому пункту. Пусто — соберётся из шаблона.
    pub punish_reason: &'a str,
    pub category_id: Option<Uuid>,
    pub server_id: Option<Uuid>,
}

pub async fn create_rule_category(
    pool: &PgPool,
    input: CategoryInput<'_>,
) -> Result<RuleCategoryRow> {
    let row = sqlx::query_as::<_, RuleCategoryRow>(
        "INSERT INTO rule_categories (code, name, description, parent_id, server_id, sort_order)
         VALUES ($1, $2, $3, $4, $5,
                 COALESCE((SELECT MAX(sort_order) + 1 FROM rule_categories), 0))
         RETURNING *",
    )
    .bind(input.code)
    .bind(input.name)
    .bind(input.description)
    .bind(input.parent_id)
    .bind(input.server_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update_rule_category(
    pool: &PgPool,
    id: Uuid,
    input: CategoryInput<'_>,
) -> Result<RuleCategoryRow> {
    let row = sqlx::query_as::<_, RuleCategoryRow>(
        "UPDATE rule_categories
         SET code = $2, name = $3, description = $4, parent_id = $5, server_id = $6,
             updated_at = NOW()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(input.code)
    .bind(input.name)
    .bind(input.description)
    .bind(input.parent_id)
    .bind(input.server_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_rule_category(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM rule_categories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Раздел не может стать собственным потомком: дерево с петлёй не отрисовать,
/// а обнаружится это уже на странице правил, пустой у всех сразу.
pub async fn category_has_ancestor(pool: &PgPool, id: Uuid, candidate: Uuid) -> Result<bool> {
    let hit: Option<(Uuid,)> = sqlx::query_as(
        "WITH RECURSIVE chain AS (
             SELECT id, parent_id FROM rule_categories WHERE id = $2
             UNION ALL
             SELECT c.id, c.parent_id FROM rule_categories c JOIN chain ON c.id = chain.parent_id
         )
         SELECT id FROM chain WHERE id = $1 LIMIT 1",
    )
    .bind(id)
    .bind(candidate)
    .fetch_optional(pool)
    .await?;
    Ok(hit.is_some())
}

pub async fn create_rule(pool: &PgPool, input: RuleInput<'_>) -> Result<RuleRow> {
    let row = sqlx::query_as::<_, RuleRow>(
        "INSERT INTO rules (code, title, description, punish_reason, category_id, server_id, sort_order)
         VALUES ($1, $2, $3, $4, $5, $6,
                 COALESCE((SELECT MAX(sort_order) + 1 FROM rules), 0))
         RETURNING *",
    )
    .bind(input.code)
    .bind(input.title)
    .bind(input.description)
    .bind(input.punish_reason)
    .bind(input.category_id)
    .bind(input.server_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update_rule(pool: &PgPool, id: Uuid, input: RuleInput<'_>) -> Result<RuleRow> {
    let row = sqlx::query_as::<_, RuleRow>(
        "UPDATE rules
         SET code = $2, title = $3, description = $4, punish_reason = $5,
             category_id = $6, server_id = $7, updated_at = NOW()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(input.code)
    .bind(input.title)
    .bind(input.description)
    .bind(input.punish_reason)
    .bind(input.category_id)
    .bind(input.server_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_rule(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM rules WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Порядок задаётся списком id целиком: так соседи не разъезжаются от
/// одиночного «поднять выше», как это было бы с обменом двух значений.
pub async fn reorder_rules(pool: &PgPool, order: &[Uuid]) -> Result<()> {
    reorder(pool, "rules", order).await
}

pub async fn reorder_rule_categories(pool: &PgPool, order: &[Uuid]) -> Result<()> {
    reorder(pool, "rule_categories", order).await
}

async fn reorder(pool: &PgPool, table: &str, order: &[Uuid]) -> Result<()> {
    // `table` не приходит извне — это литерал из двух функций выше.
    let sql = format!(
        "UPDATE {table} SET sort_order = data.pos, updated_at = NOW()
         FROM (SELECT unnest($1::uuid[]) AS id,
                      generate_subscripts($1::uuid[], 1) AS pos) AS data
         WHERE {table}.id = data.id"
    );
    sqlx::query(&sql).bind(order).execute(pool).await?;
    Ok(())
}
