//! Registry of agents connected for punishments.
//!
//! Separate from `WrapperHub`: the wrapper supervises the server process from
//! outside and only speaks console, while the agent runs inside the game and
//! knows players.
//!
//! Keyed by connection id rather than game server, because a server may
//! reconnect and there is nothing here to evict a live connection with a dead
//! one — unlike the wrapper, this channel has no "answer me" requests.

use super::proto::ToAgent;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub type ConnId = u64;

struct Conn {
    /// The build this game server belongs to; a build-wide mute or ban only
    /// reaches its own servers.
    server_id: Uuid,
    /// The game server itself — used to address a single server and to track
    /// its online roster.
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

    /// Asked on disconnect: the online roster may only be cleared once the last
    /// agent for this server is gone.
    pub fn has_game_server(&self, game_server_id: Uuid) -> bool {
        self.conns
            .iter()
            .any(|conn| conn.game_server_id == game_server_id)
    }

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

    pub fn broadcast(&self, msg: &ToAgent) {
        self.send(msg, None);
    }

    /// Send to the agents of one build, or to every agent when `None`.
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
        // The frame is built from our own types, so getting here means a bug.
        Err(e) => {
            tracing::error!(error = %e, "failed to encode agent frame");
            None
        }
    }
}
