//! Монотонные счётчики для дедупликации/упорядочивания сообщений между сторонами.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub type Serial = u64;

#[derive(Default, Debug)]
pub struct AtomicSerialProvider(Arc<AtomicU64>);

impl Clone for AtomicSerialProvider {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl AtomicSerialProvider {
    pub fn next(&self) -> Serial {
        self.0.fetch_add(1, Ordering::SeqCst) + 1
    }
}

#[derive(Default, Clone, Debug)]
pub struct AtomicSetSerial(Arc<AtomicU64>);

impl AtomicSetSerial {
    pub fn get(&self) -> Serial {
        self.0.load(Ordering::Acquire)
    }

    pub fn set(&self, serial: Serial) {
        self.0.store(serial, Ordering::Release);
    }
}
