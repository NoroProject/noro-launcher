//! Настройки инстанса и состояние первичной установки.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct InstanceState {
    pub setup_completed: bool,
    pub setup_token_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub async fn instance_state(pool: &PgPool) -> Result<InstanceState> {
    Ok(sqlx::query_as::<_, InstanceState>(
        "SELECT setup_completed, setup_token_hash, created_at, completed_at
         FROM instance_state WHERE id = TRUE",
    )
    .fetch_one(pool)
    .await?)
}

/// Все настройки одним запросом: конфиг читается целиком при старте, ходить в
/// БД по ключу незачем.
pub async fn all_settings(pool: &PgPool) -> Result<BTreeMap<String, Value>> {
    let rows = sqlx::query_as::<_, (String, Value)>("SELECT key, value FROM instance_settings")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().collect())
}

/// Одна настройка по ключу. Для тех значений, что читают на ходу, а не при
/// старте: шаблоны сообщений правят из админки, и перечитывать ради них всю
/// таблицу незачем.
pub async fn get_setting(pool: &PgPool, key: &str) -> Result<Option<Value>> {
    Ok(
        sqlx::query_scalar::<_, Value>("SELECT value FROM instance_settings WHERE key = $1")
            .bind(key)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn set_setting(pool: &PgPool, key: &str, value: &Value, by: Option<Uuid>) -> Result<()> {
    sqlx::query(
        "INSERT INTO instance_settings (key, value, updated_by)
         VALUES ($1, $2, $3)
         ON CONFLICT (key) DO UPDATE
           SET value = $2, updated_by = $3, updated_at = NOW()",
    )
    .bind(key)
    .bind(value)
    .bind(by)
    .execute(pool)
    .await?;
    Ok(())
}

/// Записать несколько настроек разом. Транзакция здесь не про
/// производительность: половина применённых URL — это инстанс, который
/// отвечает по одному адресу и редиректит на другой.
pub async fn set_settings(
    pool: &PgPool,
    values: &BTreeMap<String, Value>,
    by: Option<Uuid>,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    for (key, value) in values {
        sqlx::query(
            "INSERT INTO instance_settings (key, value, updated_by)
             VALUES ($1, $2, $3)
             ON CONFLICT (key) DO UPDATE
               SET value = $2, updated_by = $3, updated_at = NOW()",
        )
        .bind(key)
        .bind(value)
        .bind(by)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Сохранить хеш одноразового setup-токена.
pub async fn set_setup_token_hash(pool: &PgPool, hash: Option<&str>) -> Result<()> {
    sqlx::query("UPDATE instance_state SET setup_token_hash = $1 WHERE id = TRUE")
        .bind(hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// Завершить настройку: токен сжигается вместе с флагом, одним запросом.
pub async fn complete_setup(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "UPDATE instance_state
         SET setup_completed = TRUE, completed_at = NOW(), setup_token_hash = NULL
         WHERE id = TRUE",
    )
    .execute(pool)
    .await?;
    Ok(())
}
