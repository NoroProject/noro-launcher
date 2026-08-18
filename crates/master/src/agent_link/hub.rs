//! Реестр агентов, подключённых за наказаниями.
//!
//! Отдельно от `WrapperHub`: враппер — это процесс-надзиратель снаружи сервера,
//! он есть не везде и умеет только консоль. Агент живёт внутри игры и знает
//! игроков, а без своего канала узнавал бы о бане игрока не раньше следующего
//! входа — то есть уже после того, как тот дописал в чат.
//!
//! Ключ — id соединения, а не игрового сервера: один сервер сборки может
//! переподключаться, и вытеснять живое соединение мёртвым здесь нечем — в
//! отличие от враппера, адресных запросов «ответь мне» тут нет.

use super::proto::ToAgent;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub type ConnId = u64;

struct Conn {
    /// Сборка, к которой относится игровой сервер: мут и бан на сборке идут
    /// только её серверам.
    server_id: Uuid,
    /// Сам игровой сервер: по нему адресуются кадры «этому серверу» и по нему
    /// же ведётся состав онлайна.
    game_server_id: Uuid,
    tx: UnboundedSender<String>,
}

#[derive(Clone, Default)]
pub struct AgentHub {
    conns: Arc<DashMap<ConnId, Conn>>,
    next_id: Arc<AtomicU64>,
}

impl AgentHub {
    pub fn register(
        &self,
        server_id: Uuid,
        game_server_id: Uuid,
        tx: UnboundedSender<String>,
    ) -> ConnId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.conns.insert(
            id,
            Conn {
                server_id,
                game_server_id,
                tx,
            },
        );
        id
    }

    pub fn unregister(&self, id: ConnId) {
        self.conns.remove(&id);
    }

    /// Остался ли ещё хоть один живой агент этого игрового сервера. Спрашивают
    /// при обрыве: чистить состав онлайна можно только когда ушёл последний.
    pub fn has_game_server(&self, game_server_id: Uuid) -> bool {
        self.conns
            .iter()
            .any(|conn| conn.game_server_id == game_server_id)
    }

    /// Кадр одному игровому серверу — тому, где сидит адресат.
    pub fn send_to_game_server(&self, msg: &ToAgent, game_server_id: Uuid) {
        let Some(frame) = encode(msg) else {
            return;
        };
        for conn in self.conns.iter() {
            if conn.game_server_id == game_server_id {
                let _ = conn.tx.send(frame.clone());
            }
        }
    }

    pub fn connected_count(&self) -> usize {
        self.conns.len()
    }

    /// Разослать всем агентам сети.
    pub fn broadcast(&self, msg: &ToAgent) {
        self.send(msg, None);
    }

    /// Разослать агентам одной сборки. `None` — всем.
    pub fn send(&self, msg: &ToAgent, server_id: Option<Uuid>) {
        let Some(frame) = encode(msg) else {
            return;
        };
        for conn in self.conns.iter() {
            if server_id.is_none_or(|id| conn.server_id == id) {
                let _ = conn.tx.send(frame.clone());
            }
        }
    }
}

fn encode(msg: &ToAgent) -> Option<String> {
    match serde_json::to_string(msg) {
        Ok(frame) => Some(frame),
        // Кадр собирается из наших же типов: сюда можно попасть только ошибкой
        // в коде, и глотать её молча нельзя.
        Err(e) => {
            tracing::error!(error = %e, "не удалось собрать кадр для агента");
            None
        }
    }
}
