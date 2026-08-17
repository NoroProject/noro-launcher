//! Переводы содержимого свода: заголовки и текст по локалям.
//!
//! Базовая формулировка в `rules`/`rule_categories` — это фолбэк. Перевода на
//! нужный язык может не быть, и тогда игрок должен увидеть исходный текст, а
//! не пустое место.

use super::{RuleCategoryRow, RuleRow, RuleSanctionRow};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RuleTranslationRow {
    pub rule_id: Uuid,
    pub locale: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub punish_reason: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CategoryTranslationRow {
    pub category_id: Uuid,
    pub locale: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SanctionTranslationRow {
    pub sanction_id: Uuid,
    pub locale: String,
    pub label: String,
}

/// Что присылает редактор: один язык одного правила.
#[derive(Debug, Clone, Deserialize)]
pub struct TranslationInput {
    pub locale: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// Формулировка наказания на этом языке. Игрок читает её в бане.
    #[serde(default)]
    pub punish_reason: String,
}

pub async fn rule_translations(pool: &PgPool, locale: &str) -> Result<Vec<RuleTranslationRow>> {
    Ok(
        sqlx::query_as::<_, RuleTranslationRow>(
            "SELECT * FROM rule_translations WHERE locale = $1",
        )
        .bind(locale)
        .fetch_all(pool)
        .await?,
    )
}

pub async fn category_translations(
    pool: &PgPool,
    locale: &str,
) -> Result<Vec<CategoryTranslationRow>> {
    Ok(sqlx::query_as::<_, CategoryTranslationRow>(
        "SELECT * FROM rule_category_translations WHERE locale = $1",
    )
    .bind(locale)
    .fetch_all(pool)
    .await?)
}

pub async fn sanction_translations(
    pool: &PgPool,
    locale: &str,
) -> Result<Vec<SanctionTranslationRow>> {
    Ok(sqlx::query_as::<_, SanctionTranslationRow>(
        "SELECT * FROM rule_sanction_translations WHERE locale = $1",
    )
    .bind(locale)
    .fetch_all(pool)
    .await?)
}

/// Все переводы одного правила — редактору, чтобы показать языки вкладками.
pub async fn translations_of_rule(pool: &PgPool, rule_id: Uuid) -> Result<Vec<RuleTranslationRow>> {
    Ok(sqlx::query_as::<_, RuleTranslationRow>(
        "SELECT * FROM rule_translations WHERE rule_id = $1 ORDER BY locale",
    )
    .bind(rule_id)
    .fetch_all(pool)
    .await?)
}

/// То же для раздела: вкладки языков в его модалке заполняются отсюда.
pub async fn translations_of_category(
    pool: &PgPool,
    category_id: Uuid,
) -> Result<Vec<CategoryTranslationRow>> {
    Ok(sqlx::query_as::<_, CategoryTranslationRow>(
        "SELECT * FROM rule_category_translations WHERE category_id = $1 ORDER BY locale",
    )
    .bind(category_id)
    .fetch_all(pool)
    .await?)
}

/// Набор переводов правила переписывается целиком — как и санкции.
/// Пустой заголовок означает «перевода нет», и строка не сохраняется: иначе
/// фолбэк перекрывался бы пустотой.
pub async fn replace_rule_translations(
    pool: &PgPool,
    rule_id: Uuid,
    items: &[TranslationInput],
) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM rule_translations WHERE rule_id = $1")
        .bind(rule_id)
        .execute(&mut *tx)
        .await?;
    for item in items.iter().filter(|i| !i.title.trim().is_empty()) {
        sqlx::query(
            "INSERT INTO rule_translations (rule_id, locale, title, description, punish_reason)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (rule_id, locale) DO UPDATE
                 SET title = EXCLUDED.title, description = EXCLUDED.description,
                     punish_reason = EXCLUDED.punish_reason",
        )
        .bind(rule_id)
        .bind(item.locale.trim())
        .bind(item.title.trim())
        .bind(item.description.trim())
        .bind(item.punish_reason.trim())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn replace_category_translations(
    pool: &PgPool,
    category_id: Uuid,
    items: &[TranslationInput],
) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM rule_category_translations WHERE category_id = $1")
        .bind(category_id)
        .execute(&mut *tx)
        .await?;
    for item in items.iter().filter(|i| !i.title.trim().is_empty()) {
        sqlx::query(
            "INSERT INTO rule_category_translations (category_id, locale, name, description)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (category_id, locale) DO UPDATE
                 SET name = EXCLUDED.name, description = EXCLUDED.description",
        )
        .bind(category_id)
        .bind(item.locale.trim())
        .bind(item.title.trim())
        .bind(item.description.trim())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Наложить переводы на свод. Отсутствующий перевод оставляет исходный текст.
pub fn apply_locale(
    categories: &mut [RuleCategoryRow],
    rules: &mut [RuleRow],
    sanctions: &mut [RuleSanctionRow],
    cat_tr: &[CategoryTranslationRow],
    rule_tr: &[RuleTranslationRow],
    sanction_tr: &[SanctionTranslationRow],
) {
    for category in categories.iter_mut() {
        if let Some(tr) = cat_tr.iter().find(|t| t.category_id == category.id) {
            category.name = tr.name.clone();
            if !tr.description.is_empty() {
                category.description = tr.description.clone();
            }
        }
    }
    for rule in rules.iter_mut() {
        if let Some(tr) = rule_tr.iter().find(|t| t.rule_id == rule.id) {
            rule.title = tr.title.clone();
            if !tr.description.is_empty() {
                rule.description = tr.description.clone();
            }
            // Пустой перевод не затирает исходную формулировку: лучше причина
            // на языке свода, чем пустая строка в бане.
            if !tr.punish_reason.is_empty() {
                rule.punish_reason = tr.punish_reason.clone();
            }
        }
    }
    for sanction in sanctions.iter_mut() {
        if let Some(tr) = sanction_tr.iter().find(|t| t.sanction_id == sanction.id) {
            sanction.label = tr.label.clone();
        }
    }
}

#[cfg(test)]
#[path = "translations_tests.rs"]
mod tests;
