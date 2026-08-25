//! Восстановление мастера из архива.
//!
//! Каждый шаг умеет отказать до того, как что-то тронуто: сперва проверка,
//! потом распаковка рядом, и только затем подмена. Пока архив не развёрнут
//! целиком, текущие данные лежат нетронутыми.
//!
//! База заливается под живым мастером, поэтому процесс сразу после этого
//! выходит: `restart: unless-stopped` поднимет его уже на восстановленных
//! данных. Промежуточного «восстановить на живой» не бывает — пул смотрит в
//! таблицы, которых после заливки уже нет, а кэши в памяти помнят прежнюю базу.

use super::{db_load, swap, Verdict};
use crate::audit::Actor;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

/// Проверить архив и применить его. Возвращает то, что стоит показать админу
/// перед тем, как мастер уйдёт на перезапуск.
pub async fn apply(state: &AppState, archive: PathBuf, actor: &Actor) -> AppResult<Value> {
    let verdict = check(state, archive.clone()).await?;
    if let Some(reason) = verdict.refusal() {
        return Err(AppError::BadRequest(reason));
    }

    let stamp = super::stamp(chrono::Utc::now());
    let data_dir = state.config.data_dir.clone();
    let work = super::work_dir(&data_dir);

    let dump = {
        let stamp = stamp.clone();
        tokio::task::spawn_blocking(move || -> Result<PathBuf> {
            let (staged, dump) = swap::extract(&archive, &work, &stamp)?;
            swap::swap(&data_dir, &staged, &stamp)?;
            Ok(dump)
        })
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .map_err(AppError::Other)?
    };

    db_load::load(&state.config.database_url, &dump)
        .await
        .map_err(AppError::Other)?;
    let _ = tokio::fs::remove_file(&dump).await;

    db_load::record(&state.config.database_url, actor, &verdict, &stamp).await;
    schedule_exit();

    Ok(json!({
        "ok": true,
        "restored_from": verdict.meta.created_at,
        "schema_version": verdict.meta.schema_version,
        "data_files": verdict.meta.data_files,
        "previous_data": swap::old_label(&stamp),
        "restarting": true,
    }))
}

/// Проверка архива — та же, что отдаёт `inspect`. Читает файл целиком, поэтому
/// уходит в блокирующий пул.
pub async fn check(state: &AppState, archive: PathBuf) -> AppResult<Verdict> {
    let key = state.signer.verifying_key();
    let known = crate::db::known_schema_version();
    tokio::task::spawn_blocking(move || super::inspect::verify(&archive, &key, known))
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .map_err(|e| AppError::BadRequest(format!("{e:#}")))
}

/// Выйти, дав ответу уйти клиенту: второго шанса рассказать, чем всё кончилось,
/// не будет. Код 0 — штатный выход, `restart: unless-stopped` поднимет мастер
/// заново уже на восстановленных данных.
fn schedule_exit() {
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(2)).await;
        tracing::warn!(
            "восстановление завершено — мастер выходит, чтобы подняться на новых данных"
        );
        std::process::exit(0);
    });
}
