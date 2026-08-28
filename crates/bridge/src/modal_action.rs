//! ModalAction — разделяемый прогресс для долгих задач (синхронизация, обновление).
//!
//! Frontend создаёт `ModalAction`, кладёт его в сообщение к backend, и затем
//! читает прогресс прямо из `Arc` (для отрисовки модалки), пока backend его обновляет.

use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct ModalProgress {
    /// Заголовок модалки ("Синхронизация HiTech").
    pub title: String,
    /// Текущая стадия ("Скачивание ресурсов...").
    pub stage: String,
    /// Деталь (имя файла или "487 / 1203").
    pub detail: String,
    /// Сделано / всего в байтах (или штуках — зависит от стадии).
    pub done: u64,
    pub total: u64,
    /// Завершено успешно.
    pub finished: bool,
    /// Ошибка, если задача упала.
    pub error: Option<String>,
}

impl ModalProgress {
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.done as f32 / self.total as f32).clamp(0.0, 1.0)
        }
    }
}

/// Разделяемая ручка прогресса. Клонируется в обе стороны bridge.
#[derive(Clone, Default)]
pub struct ModalAction {
    inner: Arc<Mutex<ModalProgress>>,
    cancelled: Arc<AtomicBool>,
}

impl ModalAction {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ModalProgress {
                title: title.into(),
                ..Default::default()
            })),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Снимок прогресса для отрисовки.
    pub fn snapshot(&self) -> ModalProgress {
        self.inner.lock().clone()
    }

    pub fn set_stage(&self, stage: impl Into<String>) {
        let mut g = self.inner.lock();
        g.stage = stage.into();
        g.detail.clear();
    }

    pub fn set_detail(&self, detail: impl Into<String>) {
        self.inner.lock().detail = detail.into();
    }

    pub fn set_progress(&self, done: u64, total: u64) {
        let mut g = self.inner.lock();
        g.done = done;
        g.total = total;
    }

    pub fn add_done(&self, delta: u64) {
        self.inner.lock().done += delta;
    }

    pub fn finish(&self) {
        self.inner.lock().finished = true;
    }

    pub fn fail(&self, error: impl Into<String>) {
        self.inner.lock().error = Some(error.into());
    }

    /// Запросить отмену задачи (нажатие "Отмена" в модалке).
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl std::fmt::Debug for ModalAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModalAction")
            .field("progress", &self.snapshot())
            .finish()
    }
}
