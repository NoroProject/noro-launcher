//! Слой доступа к БД. Используются runtime-проверяемые запросы (sqlx::query*),
//! т.к. компиляция идёт без живой БД.

pub mod audit;
pub mod build_copy;
pub mod capes;
pub mod cleanup;
pub mod game_servers;
pub mod integrity;
pub mod launcher_clients;
pub mod mod_suggestions;
pub mod models;
pub mod oauth2;
pub mod passkeys;
pub mod queries;

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub use audit::*;
pub use build_copy::*;
pub use capes::*;
pub use game_servers::*;
pub use integrity::*;
pub use launcher_clients::*;
pub use mod_suggestions::*;
pub use oauth2::*;
pub use passkeys::*;
pub use queries::*;

/// Подключиться к БД и применить миграции.
pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(database_url)
        .await?;
    // Ошибку миграций нельзя проглатывать: она обрывает всю дальнейшую цепочку,
    // и мастер поднимется на схеме, которой не соответствует код.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("применение миграций")?;
    tracing::info!("миграции применены");
    Ok(pool)
}

#[cfg(test)]
#[path = "migrations_tests.rs"]
mod tests;
