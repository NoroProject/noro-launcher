//! Операции над игровым сервером поверх канала враппера.
//!
//! Тонкий слой над `WrapperHub::call`: тип аргументов и разбор ответа. Смысл в
//! том, чтобы имена операций и форма их результата жили в одном месте, а не
//! разъезжались по хендлерам.

use super::proto::Op;
use crate::catalog::ResolvedMod;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use uuid::Uuid;

/// `start` | `stop` | `restart` | `kill`.
pub async fn power(state: &AppState, server: Uuid, action: &str) -> AppResult<()> {
    if !matches!(action, "start" | "stop" | "restart" | "kill") {
        return Err(AppError::BadRequest(format!(
            "unknown power action: {action}"
        )));
    }
    state
        .wrappers
        .call(
            server,
            Op::Power {
                action: action.into(),
            },
        )
        .await?;
    Ok(())
}

pub async fn command(state: &AppState, server: Uuid, line: &str) -> AppResult<()> {
    let line = line.trim();
    if line.is_empty() {
        return Err(AppError::BadRequest("empty command".into()));
    }
    state
        .wrappers
        .call(
            server,
            Op::Command {
                line: line.to_string(),
            },
        )
        .await?;
    Ok(())
}

/// Куда платформа кладёт расширения. Paper и прокси зовут это плагинами, все
/// остальные — модами; папка одна, а имя разное.
pub fn mod_dir(platform: &str) -> &'static str {
    match platform {
        "paper" | "bukkit" | "spigot" | "purpur" | "velocity" | "bungeecord" => "plugins",
        _ => "mods",
    }
}

/// Поставить мод на игровой сервер.
///
/// Файл уже лежит в сторе мастера — врапперу уходит ссылка и sha1, качает он
/// сам. Гонять jar через WebSocket ради того же результата смысла нет, а стор
/// уже умеет докачку и кеширование.
pub async fn install_mod(
    state: &AppState,
    server: Uuid,
    resolved: &ResolvedMod,
) -> AppResult<String> {
    let conn = state.wrappers.require(server)?;
    let platform = conn
        .state()
        .info
        .map(|info| info.platform)
        .unwrap_or_default();
    let dir = mod_dir(&platform);

    conn.call(Op::ModInstall {
        url: state.config.file_url(&resolved.sha1),
        sha1: resolved.sha1.clone(),
        filename: resolved.filename.clone(),
        dir: dir.to_string(),
    })
    .await?;
    Ok(format!("{dir}/{}", resolved.filename))
}

pub async fn backup_create(
    state: &AppState,
    server: Uuid,
    name: &str,
) -> AppResult<serde_json::Value> {
    state
        .wrappers
        .call(
            server,
            Op::BackupCreate {
                name: name.to_string(),
            },
        )
        .await
}

pub async fn backup_list(state: &AppState, server: Uuid) -> AppResult<serde_json::Value> {
    state.wrappers.call(server, Op::BackupList).await
}

pub async fn backup_restore(
    state: &AppState,
    server: Uuid,
    name: &str,
) -> AppResult<serde_json::Value> {
    state
        .wrappers
        .call(
            server,
            Op::BackupRestore {
                name: name.to_string(),
            },
        )
        .await
}

pub async fn backup_delete(
    state: &AppState,
    server: Uuid,
    name: &str,
) -> AppResult<serde_json::Value> {
    state
        .wrappers
        .call(
            server,
            Op::BackupDelete {
                name: name.to_string(),
            },
        )
        .await
}
