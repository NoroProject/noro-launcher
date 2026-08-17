//! Санкции правила: допустимые виды наказаний и рамки сроков.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RuleSanctionRow {
    pub id: Uuid,
    pub rule_id: Uuid,
    /// `warn` | `mute` | `ban` | `server_ban`.
    pub kind: String,
    pub label: String,
    /// `None` — без нижней границы.
    pub min_minutes: Option<i64>,
    /// `None` — допустимо вплоть до «навсегда».
    pub max_minutes: Option<i64>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SanctionInput {
    pub kind: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub min_minutes: Option<i64>,
    #[serde(default)]
    pub max_minutes: Option<i64>,
}

impl RuleSanctionRow {
    /// Попадает ли срок в рамки варианта. `None` — «навсегда»: оно проходит
    /// только там, где верхняя граница не задана.
    pub fn allows(&self, minutes: Option<i64>) -> bool {
        if self.kind == "warn" {
            return true;
        }
        match minutes {
            None => self.max_minutes.is_none(),
            Some(value) => {
                self.min_minutes.is_none_or(|min| value >= min)
                    && self.max_minutes.is_none_or(|max| value <= max)
            }
        }
    }
}

/// Санкции нескольких правил разом: страница правил рисует их все, и запрос на
/// каждое правило превращал бы открытие свода в сотню обращений к базе.
pub async fn list_rule_sanctions(
    pool: &PgPool,
    rule_ids: &[Uuid],
) -> Result<Vec<RuleSanctionRow>> {
    let rows = sqlx::query_as::<_, RuleSanctionRow>(
        "SELECT * FROM rule_sanctions WHERE rule_id = ANY($1) ORDER BY sort_order, kind",
    )
    .bind(rule_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_all_rule_sanctions(pool: &PgPool) -> Result<Vec<RuleSanctionRow>> {
    let rows = sqlx::query_as::<_, RuleSanctionRow>(
        "SELECT * FROM rule_sanctions ORDER BY sort_order, kind",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn sanctions_of_rule(pool: &PgPool, rule_id: Uuid) -> Result<Vec<RuleSanctionRow>> {
    let rows = sqlx::query_as::<_, RuleSanctionRow>(
        "SELECT * FROM rule_sanctions WHERE rule_id = $1 ORDER BY sort_order, kind",
    )
    .bind(rule_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Набор переписывается целиком: редактор правила присылает готовый список, и
/// сверять его построчно значило бы держать две версии одной правды.
pub async fn replace_rule_sanctions(
    pool: &PgPool,
    rule_id: Uuid,
    items: &[SanctionInput],
) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM rule_sanctions WHERE rule_id = $1")
        .bind(rule_id)
        .execute(&mut *tx)
        .await?;
    for (index, item) in items.iter().enumerate() {
        sqlx::query(
            "INSERT INTO rule_sanctions (rule_id, kind, label, min_minutes, max_minutes, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(rule_id)
        .bind(&item.kind)
        .bind(item.label.trim())
        // Предупреждение не имеет длительности, и сохранённые рамки только
        // сбивали бы с толку в форме модерации.
        .bind(if item.kind == "warn" { None } else { item.min_minutes })
        .bind(if item.kind == "warn" { None } else { item.max_minutes })
        .bind(index as i32)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
#[path = "sanctions_tests.rs"]
mod tests;
