//! Слой доступа к БД. Используются runtime-проверяемые запросы (sqlx::query*),
//! т.к. компиляция идёт без живой БД.

pub mod audit;
pub mod blocklist;
pub mod build_copy;
pub mod capes;
pub mod cleanup;
pub mod freezes;
pub mod game_servers;
pub mod game_sessions;
pub mod impersonation;
pub mod instance;
pub mod integrity;
pub mod launcher_clients;
pub mod local_account;
pub mod log_requests;
pub mod mod_suggestions;
pub mod models;
pub mod notes;
pub mod oauth2;
pub mod passkeys;
pub mod punishments;
pub mod queries;
pub mod reports;
pub mod restart_schedules;
pub mod rules;
pub mod sessions;
pub mod support;

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub use audit::*;
pub use blocklist::*;
pub use build_copy::*;
pub use capes::*;
pub use freezes::*;
pub use game_servers::*;
pub use game_sessions::*;
pub use impersonation::*;
pub use instance::*;
pub use integrity::*;
pub use launcher_clients::*;
pub use local_account::*;
pub use log_requests::*;
pub use mod_suggestions::*;
pub use notes::*;
pub use oauth2::*;
pub use passkeys::*;
pub use punishments::*;
pub use queries::*;
pub use reports::*;
pub use restart_schedules::*;
pub use rules::*;
pub use sessions::*;
pub use support::*;

/// Подключиться к БД и применить миграции.
pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(database_url)
        .await?;

    let migrator = sqlx::migrate!("./migrations");
    if let Err(e) = migrator.run(&pool).await {
        let err_str = e.to_string();
        if err_str.contains("was previously applied but has been modified") {
            tracing::warn!(error = %e, "обнаружена изменённая миграция на диске, обновляем контрольные суммы в _sqlx_migrations...");
            for m in migrator.migrations.iter() {
                let _ = sqlx::query(
                    "UPDATE _sqlx_migrations SET checksum = $1 WHERE version = $2",
                )
                .bind(m.checksum.as_ref())
                .bind(m.version)
                .execute(&pool)
                .await;
            }
            migrator
                .run(&pool)
                .await
                .context("применение миграций (после автообновления checksum)")?;
        } else {
            return Err(e).context("применение миграций");
        }
    }

    tracing::info!("миграции применены");
    Ok(pool)
}

#[cfg(test)]
#[path = "migrations_tests.rs"]
mod tests;
