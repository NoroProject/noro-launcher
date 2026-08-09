//! Реестр подключённых врапперов и вызовы к ним.
//!
//! Ключ — id игрового сервера: враппер на нём один, и мастер обязан уметь
//! адресовать его точно. Соединение живёт в `Arc`, чтобы вызов мог отпустить
//! ссылку на карту до `await` — держать guard шардированной карты через точку
//! ожидания значит однажды встать намертво.

use super::proto::{Op, WrapperInfo, WrapperState, WrapperStatus};
use crate::error::{AppError, AppResult};
use dashmap::DashMap;
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;

/// Сколько строк консоли помним для тех, кто откроет её позже.
const BACKLOG: usize = 500;

pub struct Conn {
    tx: mpsc::UnboundedSender<String>,
    pending: DashMap<u64, oneshot::Sender<Result<Value, String>>>,
    next_id: AtomicU64,
    console: broadcast::Sender<String>,
    backlog: Mutex<VecDeque<String>>,
    info: WrapperInfo,
    status: Mutex<WrapperStatus>,
}

impl Conn {
    pub async fn call(&self, op: Op) -> AppResult<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (reply_tx, reply_rx) = oneshot::channel();
        self.pending.insert(id, reply_tx);

        let mut frame = serde_json::to_value(&op).map_err(|e| AppError::Other(e.into()))?;
        frame["type"] = json!("request");
        frame["id"] = json!(id);
        if self.tx.send(frame.to_string()).is_err() {
            self.pending.remove(&id);
            return Err(AppError::BadRequest("враппер отключился".into()));
        }

        match tokio::time::timeout(op.timeout(), reply_rx).await {
            Ok(Ok(Ok(data))) => Ok(data),
            Ok(Ok(Err(message))) => Err(AppError::BadRequest(message)),
            Ok(Err(_)) => Err(AppError::BadRequest("соединение с враппером закрылось".into())),
            Err(_) => {
                // Ответ уже не придёт вовремя — снимаем ожидание, иначе карта
                // ожиданий растёт на каждый зависший вызов.
                self.pending.remove(&id);
                Err(AppError::BadRequest("враппер не ответил вовремя".into()))
            }
        }
    }

    pub fn resolve(&self, id: u64, result: Result<Value, String>) {
        if let Some((_, waiter)) = self.pending.remove(&id) {
            let _ = waiter.send(result);
        }
    }

    pub fn push_console(&self, line: String) {
        let mut backlog = self.backlog.lock();
        if backlog.len() == BACKLOG {
            backlog.pop_front();
        }
        backlog.push_back(line.clone());
        drop(backlog);
        // Ошибка значит «никто не смотрит» — это норма, а не сбой.
        let _ = self.console.send(line);
    }

    pub fn backlog(&self) -> Vec<String> {
        self.backlog.lock().iter().cloned().collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.console.subscribe()
    }

    pub fn set_status(&self, status: WrapperStatus) {
        *self.status.lock() = status;
    }

    pub fn state(&self) -> WrapperState {
        WrapperState {
            connected: true,
            info: Some(self.info.clone()),
            status: self.status.lock().clone(),
        }
    }
}

#[derive(Clone, Default)]
pub struct WrapperHub {
    conns: Arc<DashMap<Uuid, Arc<Conn>>>,
}

impl WrapperHub {
    pub fn register(
        &self,
        game_server_id: Uuid,
        info: WrapperInfo,
        status: WrapperStatus,
        tx: mpsc::UnboundedSender<String>,
    ) -> Arc<Conn> {
        let conn = Arc::new(Conn {
            tx,
            pending: DashMap::new(),
            next_id: AtomicU64::new(1),
            console: broadcast::channel(256).0,
            backlog: Mutex::new(VecDeque::with_capacity(BACKLOG)),
            info,
            status: Mutex::new(status),
        });
        // Переподключение вытесняет прежнее соединение: живой враппер на сервере
        // один, а зависший сокет иначе остался бы адресатом команд.
        self.conns.insert(game_server_id, conn.clone());
        conn
    }

    /// Снять регистрацию, только если она всё ещё наша: пока сокет умирал,
    /// враппер мог успеть переподключиться, и чужое соединение сносить нельзя.
    pub fn unregister(&self, game_server_id: Uuid, conn: &Arc<Conn>) {
        self.conns
            .remove_if(&game_server_id, |_, current| Arc::ptr_eq(current, conn));
    }

    pub fn get(&self, game_server_id: Uuid) -> Option<Arc<Conn>> {
        self.conns.get(&game_server_id).map(|c| c.clone())
    }

    pub fn require(&self, game_server_id: Uuid) -> AppResult<Arc<Conn>> {
        self.get(game_server_id)
            .ok_or_else(|| AppError::BadRequest("враппер этого сервера не подключён".into()))
    }

    pub fn state(&self, game_server_id: Uuid) -> WrapperState {
        self.get(game_server_id)
            .map(|c| c.state())
            .unwrap_or_else(WrapperState::offline)
    }

    pub async fn call(&self, game_server_id: Uuid, op: Op) -> AppResult<Value> {
        self.require(game_server_id)?.call(op).await
    }
}
