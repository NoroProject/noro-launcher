//! Первичная настройка инстанса.
//!
//! Визард двухфазный, и это прямое следствие решения «секреты только в env»:
//! дописать `DISCORD_CLIENT_SECRET` или `NORO_SIGNING_KEY` в чужое окружение
//! мастер физически не может. Поэтому фаза A собирает несекретное в БД и
//! показывает блок для `.env`, а фаза B — уже после рестарта — видит секреты и
//! проверяет их по-настоящему.

pub mod api;
pub mod gate;
mod token;

pub use token::{ensure_token, verify_token, SetupAuth};

use crate::config::keys;
use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::BTreeMap;

/// Перенести несекретные значения из окружения в БД и закрыть настройку.
///
/// Боевой инстанс перехода не замечает и никогда не видит `503 setup_required`:
/// у него всё уже задано в compose, и спрашивать его об этом заново незачем.
/// Признак «это работающий инстанс, а не пустой» — заданный `NORO_PUBLIC_URL`.
pub async fn migrate_env_if_needed(pool: &PgPool) -> Result<bool> {
    let state = crate::db::instance_state(pool).await?;
    if state.setup_completed {
        return Ok(false);
    }
    if crate::config::env_opt(keys::PUBLIC_URL.env).is_none() {
        return Ok(false);
    }

    let mut values: BTreeMap<String, Value> = BTreeMap::new();
    for key in keys::ALL {
        if let Some(v) = crate::config::env_opt(key.env) {
            values.insert(key.name.to_string(), Value::String(v));
        }
    }

    crate::db::set_settings(pool, &values, None).await?;
    crate::db::complete_setup(pool).await?;

    tracing::info!(
        settings = values.len(),
        "настройки перенесены из окружения в БД, первичная настройка отмечена завершённой"
    );
    Ok(true)
}
