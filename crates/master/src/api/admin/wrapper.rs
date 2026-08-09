//! Админ-API управления игровым сервером: состояние, питание, консоль.
//!
//! Все ручки — тонкая обёртка над каналом враппера. Право отдельное
//! (`noro.admin.wrapper`): рестарт и запись файлов на игровой машине — не то же
//! самое, что правка её карточки в админке.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use crate::wrapper::ops;
use crate::wrapper::proto::WrapperState;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::Stream;
use schema::PERM_ADMIN_WRAPPER;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::Infallible;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

pub async fn status(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<WrapperState>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    Ok(Json(state.wrappers.state(id)))
}

#[derive(Deserialize)]
pub struct PowerReq {
    /// `start` | `stop` | `restart` | `kill`.
    pub action: String,
}

pub async fn power(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<PowerReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    ops::power(&state, id, &req.action).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct CommandReq {
    pub line: String,
}

pub async fn command(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CommandReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    ops::command(&state, id, &req.line).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Serialize)]
pub struct ConsoleBacklog {
    pub lines: Vec<String>,
}

/// Хвост консоли. Живой поток идёт по WebSocket, а это то, что было до того,
/// как админ открыл вкладку.
pub async fn console(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ConsoleBacklog>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    let lines = state
        .wrappers
        .get(id)
        .map(|conn| conn.backlog())
        .unwrap_or_default();
    Ok(Json(ConsoleBacklog { lines }))
}

/// Живой поток консоли.
///
/// SSE, а не WebSocket: токен админки лежит в куке веб-origin и уходит мастеру
/// заголовком `Authorization`, а браузерный `WebSocket` заголовки ставить не
/// умеет. Городить передачу токена в query-строке (где он осядет в логах) ради
/// одностороннего потока текста незачем.
pub async fn console_stream(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    let rx = state.wrappers.require(id)?.subscribe();

    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(line) => return Some((Ok(Event::default().data(line)), rx)),
                // Читатель отстал: сервер сыпал в консоль быстрее, чем браузер
                // успевал. Пропуск лучше обрыва — дальше поток продолжится.
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return None,
            }
        }
    });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
