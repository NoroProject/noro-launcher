//! Реестр открытых вкладок админки.
//!
//! Отдельный от лаунчерного: кадры у них разные, и смешивать их значит однажды
//! отправить браузеру диалог, который должен спрашивать машину.

use dashmap::DashMap;
use schema::AdminWsMsg;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub type ConnId = u64;

struct Conn {
    /// `None`, пока вкладка не прошла `Authenticate`.
    user_id: Option<Uuid>,
    tx: UnboundedSender<AdminWsMsg>,
}

#[derive(Clone, Default)]
pub struct AdminHub {
    conns: Arc<DashMap<ConnId, Conn>>,
    next_id: Arc<AtomicU64>,
}

impl AdminHub {
    pub fn register(&self, tx: UnboundedSender<AdminWsMsg>) -> ConnId {
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

    /// Разослать всем вошедшим вкладкам.
    ///
    /// Именно всем, а не тому, кто держит дело: на карточку смотрит и тот, кто
    /// её не взял. В кадре нет данных — только id, а за самой карточкой
    /// страница идёт обычным запросом, где права и проверяются.
    pub fn broadcast(&self, msg: &AdminWsMsg) {
        for c in self.conns.iter() {
            if c.user_id.is_some() {
                let _ = c.tx.send(msg.clone());
            }
        }
    }
}
