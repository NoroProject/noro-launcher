//! Database access. Runtime-checked queries (`sqlx::query*`) throughout — the
//! build has no live database to check against.

pub mod audit;
pub mod auth_methods;
pub mod blocklist;
pub mod build_copy;
pub mod capes;
pub mod cases;
pub mod cleanup;
pub mod freezes;
pub mod game_servers;
pub mod game_sessions;
pub mod identities;
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
pub mod oauth_apps;
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
pub use cases::*;
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
pub use oauth_apps::*;
pub use passkeys::*;
pub use punishments::*;
pub use queries::*;
pub use reports::*;
pub use restart_schedules::*;
pub use rules::*;
pub use sessions::*;
pub use support::*;

pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(database_url)
        .await?;

    let migrator = sqlx::migrate!("./migrations");
    // A checksum mismatch fails the start. Do not "fix" it by rewriting
    // `_sqlx_migrations.checksum`: sqlx never re-runs an applied migration, so
    // the edit would not reach the database and it would drift from the files
    // while still reporting them as applied.
    migrator.run(&pool).await.context(
        "running migrations. A complaint about a changed migration means the \
         file was edited after it had already been applied. Put it back and add \
         a new migration instead — sqlx does not re-run applied ones, so the \
         edit will never reach the database",
    )?;

    tracing::info!("migrations applied");
    Ok(pool)
}

/// Latest migration compiled into this binary.
///
/// Restore checks this: a dump at schema 62 must not be loaded into a master
/// that only knows 58. Migrations don't roll backwards, so it would come up on
/// data it doesn't understand.
pub fn known_schema_version() -> i64 {
    sqlx::migrate!("./migrations")
        .iter()
        .map(|m| m.version)
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "migrations_tests.rs"]
mod tests;
