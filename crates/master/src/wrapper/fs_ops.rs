//! Файловые операции на игровом сервере и веер по нескольким серверам.
//!
//! Мастер здесь ничего не решает про пути: их проверяет враппер, у которого
//! есть настоящая файловая система. Дублировать проверку на две реализации —
//! верный способ развести их со временем.

use super::proto::Op;
use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

pub async fn list(state: &AppState, server: Uuid, path: &str) -> AppResult<Value> {
    state
        .wrappers
        .call(
            server,
            Op::FsList {
                path: path.to_string(),
            },
        )
        .await
}

pub async fn read(state: &AppState, server: Uuid, path: &str) -> AppResult<Value> {
    state
        .wrappers
        .call(
            server,
            Op::FsRead {
                path: path.to_string(),
            },
        )
        .await
}

pub async fn write(state: &AppState, server: Uuid, path: &str, content: &str) -> AppResult<Value> {
    state.wrappers.call(server, write_op(path, content)).await
}

pub async fn delete(state: &AppState, server: Uuid, path: &str) -> AppResult<Value> {
    state
        .wrappers
        .call(
            server,
            Op::FsDelete {
                path: path.to_string(),
            },
        )
        .await
}

pub async fn mkdir(state: &AppState, server: Uuid, path: &str) -> AppResult<Value> {
    state
        .wrappers
        .call(
            server,
            Op::FsMkdir {
                path: path.to_string(),
            },
        )
        .await
}

pub fn write_op(path: &str, content: &str) -> Op {
    Op::FsWrite {
        path: path.to_string(),
        content: content.to_string(),
    }
}

#[derive(Serialize)]
pub struct ServerOutcome {
    pub id: Uuid,
    pub ok: bool,
    pub error: Option<String>,
}

/// Одна операция на несколько серверов.
///
/// Последовательно, а не пачкой: если конфиг ломает сервер, лучше увидеть это
/// на первом, чем разослать поломку на все сразу. Отчёт построчный — упавший
/// сервер не отменяет остальные.
pub async fn fan_out(state: &AppState, servers: &[Uuid], op: Op) -> Vec<ServerOutcome> {
    let mut results = Vec::with_capacity(servers.len());
    for &id in servers {
        let outcome = state.wrappers.call(id, op.clone()).await;
        results.push(ServerOutcome {
            id,
            ok: outcome.is_ok(),
            error: outcome.err().map(|e| e.to_string()),
        });
    }
    results
}
