//! Состояние дел в лаунчере: очередь и открытая карточка.
//!
//! Живёт в беке, а не в моде, ради того, ради чего вообще делался канал:
//! UI над этим store потом будет два. Вкладка «Дела» в самом лаунчере
//! получается почти бесплатно, и модератору не нужен браузер, даже когда игра
//! закрыта.
//!
//! Игра не запущена — кадр просто некуда класть, и это нормально: store
//! переживёт до следующего подключения.

use mod_link::{CaseBrief, CaseView};
use parking_lot::Mutex;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Default)]
struct Inner {
    queue: Vec<CaseBrief>,
    /// Что за страницу мы держим: сколько дел подходит под фильтр целиком, с
    /// какого сдвига взята страница и по какому запросу. Нужно, чтобы обновить
    /// её на месте по `CaseUpdated` — иначе модератора выбрасывало бы на первую
    /// страницу каждый раз, когда кто-то тронул любое дело.
    total: i64,
    offset: i64,
    query: Option<String>,
    /// Дело, которое мод сейчас держит открытым. По нему и только по нему
    /// перечитывается карточка на `CaseUpdated`.
    open: Option<CaseView>,
}

#[derive(Clone, Default)]
pub struct CaseStore {
    inner: Arc<Mutex<Inner>>,
}

impl CaseStore {
    pub fn set_queue(&self, cases: Vec<CaseBrief>, total: i64, offset: i64, query: Option<String>) {
        let mut inner = self.inner.lock();
        inner.queue = cases;
        inner.total = total;
        inner.offset = offset;
        inner.query = query;
    }

    pub fn queue(&self) -> Vec<CaseBrief> {
        self.inner.lock().queue.clone()
    }

    /// Чем была получена текущая страница: запрос и сдвиг.
    pub fn queue_spot(&self) -> (Option<String>, i64) {
        let inner = self.inner.lock();
        (inner.query.clone(), inner.offset)
    }

    pub fn queue_total(&self) -> i64 {
        self.inner.lock().total
    }

    pub fn set_open(&self, view: CaseView) {
        self.inner.lock().open = Some(view);
    }

    pub fn close(&self) {
        self.inner.lock().open = None;
    }

    pub fn open(&self) -> Option<CaseView> {
        self.inner.lock().open.clone()
    }

    /// Открыто ли сейчас именно это дело. `CaseUpdated` приходит на любое дело
    /// за модератором, а перечитывать имеет смысл только то, что на экране.
    pub fn is_open(&self, case_id: Uuid) -> bool {
        self.inner
            .lock()
            .open
            .as_ref()
            .is_some_and(|v| v.brief.id == case_id)
    }

    pub fn open_id(&self) -> Option<Uuid> {
        self.inner.lock().open.as_ref().map(|v| v.brief.id)
    }
}
