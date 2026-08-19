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
    // Расхождение контрольной суммы — отказ, а не повод её переписать.
    //
    // Раньше здесь стояло автообновление `_sqlx_migrations.checksum` под то,
    // что лежит на диске. Оно снимало симптом и оставляло болезнь: правку уже
    // применённой миграции sqlx второй раз не выполняет, поэтому база начинала
    // расходиться с файлами, продолжая рапортовать «применено».
    //
    // Так и вышло с волной 3: `0050` поправили после применения, суммы молча
    // переписались, а `player_reports` не появилась ни на одной базе — админка
    // падала на «relation does not exist», и чинить пришлось отдельной
    // миграцией. Молчаливая расходимость дороже несостоявшегося старта: второе
    // видно сразу, первое — через неделю и не там, где сломали.
    migrator.run(&pool).await.context(
        "применение миграций. Если ругается на изменённую миграцию — файл \
         правили после того, как он уже применился. Возвращайте его как было и \
         заводите новую миграцию: sqlx не выполняет применённые повторно, и \
         правка всё равно не доедет до базы",
    )?;

    tracing::info!("миграции применены");
    Ok(pool)
}

#[cfg(test)]
#[path = "migrations_tests.rs"]
mod tests;
