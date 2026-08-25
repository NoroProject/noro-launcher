//! Заливка дампа и запись о ней в журнал.
//!
//! Обе операции идут мимо пула мастера: к моменту их выполнения он смотрит в
//! объекты, которых в базе уже нет.

use super::Verdict;
use crate::audit::{actions, Actor};
use anyhow::{bail, Context, Result};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::path::Path;

/// Полная очистка перед заливкой.
///
/// Одного `--clean` из дампа мало: он снимает только те объекты, что в дампе
/// есть. Восстанавливая архив постарее, мы оставили бы таблицы, заведённые
/// более поздними миграциями, и одновременно откатили бы `_sqlx_migrations`
/// назад — на старте мастер накатывал бы те же миграции повторно и падал на
/// «relation already exists». Проверено на схеме из 62 миграций: без этой
/// очистки посторонняя таблица переживает восстановление.
///
/// Чужие соединения обрываются первыми: пул живого мастера держит их
/// открытыми, и `DROP SCHEMA` ждал бы блокировку. Мастер всё равно выходит
/// следом, так что терять нечего. `lock_timeout` — чтобы упереться в
/// затянувшийся запрос и отказать, а не висеть бесконечно.
const KILL: &str = "SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
                    WHERE datname = current_database() AND pid <> pg_backend_pid()";
const RESET: &str = "SET lock_timeout = '30s'; DROP SCHEMA public CASCADE; CREATE SCHEMA public";

/// Залить дамп. `ON_ERROR_STOP` обязателен: без него psql проглатывает упавшие
/// операторы и выходит с нулём, оставляя базу собранной наполовину.
pub async fn load(database_url: &str, dump: &Path) -> Result<()> {
    let out = tokio::process::Command::new("psql")
        .args(["--set=ON_ERROR_STOP=1", "--quiet"])
        .args(["-c", KILL])
        .args(["-c", RESET])
        .arg("--file")
        .arg(dump)
        .arg(database_url)
        .output()
        .await
        .context("не удалось запустить psql. Он есть в образе мастера?")?;
    if !out.status.success() {
        bail!(
            "psql не залил дамп: {}. Файлы уже подменены, прежние лежат в \
             .noro-backup/old-*; база осталась в том виде, до которого дошла заливка",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

/// Записать событие после заливки, а не до неё: дамп сносит и наливает всю
/// базу, и строка, написанная раньше, исчезла бы вместе с прежним журналом.
pub async fn record(database_url: &str, actor: &Actor, verdict: &Verdict, stamp: &str) {
    let details = json!({
        "created_at": verdict.meta.created_at,
        "master_version": verdict.meta.master_version,
        "schema_version": verdict.meta.schema_version,
        "signed": verdict.meta.signed,
        "signature_ok": verdict.signature_ok,
        "data_files": verdict.meta.data_files,
        "previous_data": super::swap::old_label(stamp),
    });

    // Своё соединение: пул мастера после заливки нерабочий, и его соединения
    // psql только что оборвал.
    let pool = match PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!(error = %e, "восстановление прошло, но записать его в аудит не вышло");
            return;
        }
    };
    if let Err(e) = crate::db::insert_audit(
        &pool,
        actor.id(),
        &actor.label(),
        actions::BACKUP_RESTORE.name,
        Some("backup"),
        Some(stamp),
        &details,
        None,
    )
    .await
    {
        tracing::error!(error = %e, "восстановление прошло, но записать его в аудит не вышло");
    }
}
