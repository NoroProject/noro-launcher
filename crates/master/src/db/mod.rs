//! Слой доступа к БД. Используются runtime-проверяемые запросы (sqlx::query*),
//! т.к. компиляция идёт без живой БД.

pub mod capes;
pub mod game_servers;
pub mod mod_suggestions;
pub mod models;
pub mod oauth2;
pub mod passkeys;
pub mod queries;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub use capes::*;
pub use game_servers::*;
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
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => tracing::info!("миграции применены"),
        Err(e) => tracing::info!(error = %e, "миграции sqlx выполнены или пропущены"),
    }
    if let Err(e) = ensure_default_launcher_app(&pool).await {
        tracing::error!(error = %e, "ошибка при создании таблиц OAuth2 Провайдера");
    }
    Ok(pool)
}
