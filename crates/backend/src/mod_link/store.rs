//! Case state: the queue page and the open card.
//!
//! It lives in the backend rather than the mod because a second UI will sit on
//! top of it — a Cases tab in the launcher itself comes almost for free, and
//! then a moderator doesn't need a browser with the game closed.
//!
//! With no game running there's nowhere to send a frame, and that's fine: the
//! store keeps its contents until the next connection.

use mod_link::{CaseBrief, CaseView};
use parking_lot::Mutex;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Default)]
struct Inner {
    queue: Vec<CaseBrief>,
    /// Which page this is: matches in total, the offset it came from, and the
    /// query behind it. Needed to refresh it in place on `CaseUpdated` — without
    /// it the moderator lands back on page one whenever anyone touches any case.
    total: i64,
    offset: i64,
    query: Option<String>,
    /// The case the mod currently has open. Only this one gets its card re-read
    /// on `CaseUpdated`.
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

    /// The query and offset the current page was fetched with.
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

    /// `CaseUpdated` arrives for every case the moderator follows, but only
    /// what's on screen is worth re-reading.
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
