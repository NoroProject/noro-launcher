//! Реестр подключённых лаунчеров. Позволяет слать сообщения конкретному
//! пользователю (обновление прав) и всем сразу (деплой новой версии лаунчера).

use dashmap::DashMap;
use schema::ServerWsMsg;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub type ConnId = u64;

struct Conn {
    /// id аутентифицированного пользователя (None пока не прошёл Authenticate).
    user_id: Option<Uuid>,
    tx: UnboundedSender<ServerWsMsg>,
}

#[derive(Clone, Default)]
pub struct WsHub {
    conns: Arc<DashMap<ConnId, Conn>>,
    next_id: Arc<AtomicU64>,
}

impl WsHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// Зарегистрировать новое соединение, вернуть его id.
    pub fn register(&self, tx: UnboundedSender<ServerWsMsg>) -> ConnId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.conns.insert(id, Conn { user_id: None, tx });
        id
    }

    pub fn authenticate(&self, id: ConnId, user_id: Uuid) {
        if let Some(mut c) = self.conns.get_mut(&id) {
            c.user_id = Some(user_id);
        }
    }

    pub fn unregister(&self, id: ConnId) {
        self.conns.remove(&id);
    }

    pub fn connected_count(&self) -> usize {
        self.conns.len()
    }

    pub fn authed_users(&self) -> usize {
        self.conns.iter().filter(|c| c.user_id.is_some()).count()
    }

    /// Послать всем соединениям конкретного пользователя.
    pub fn send_to_user(&self, user_id: Uuid, msg: &ServerWsMsg) {
        for c in self.conns.iter() {
            if c.user_id == Some(user_id) {
                let _ = c.tx.send(msg.clone());
            }
        }
    }

    /// В сети ли лаунчер игрока. Нужно, чтобы понять, слать ли диалог или
    /// сразу предлагать фолбэк-код.
    pub fn is_user_connected(&self, user_id: Uuid) -> bool {
        self.conns.iter().any(|c| c.user_id == Some(user_id))
    }

    /// Разослать всем подключённым лаунчерам.
    pub fn broadcast(&self, msg: &ServerWsMsg) {
        for c in self.conns.iter() {
            let _ = c.tx.send(msg.clone());
        }
    }
}
