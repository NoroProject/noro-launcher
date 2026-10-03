//! Both ends of the channel: commands travel frontend → backend, updates come
//! back the other way. The handles clone freely, the receivers do not.
//!
//! Debug builds cap the command channel at 256 messages so a backlog is visible
//! during development. Updates are never capped or dropped, debug or not: a
//! lost update is state the window never learns — a game that has exited but
//! still offers «Stop», a build that is installed but still offers «Install».

#[cfg(debug_assertions)]
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::message::{MessageToBackend, MessageToFrontend};
use crate::serial::{AtomicSerialProvider, AtomicSetSerial, Serial};

pub fn create_pair() -> (
    BackendReceiver,
    BackendHandle,
    FrontendReceiver,
    FrontendHandle,
) {
    let (frontend_send, frontend_recv) = tokio::sync::mpsc::unbounded_channel();
    #[cfg(debug_assertions)]
    let (backend_send, backend_recv) = tokio::sync::mpsc::channel(256);
    #[cfg(not(debug_assertions))]
    let (backend_send, backend_recv) = tokio::sync::mpsc::unbounded_channel();

    let backend_serial = AtomicSetSerial::default();
    let frontend_serial = AtomicSetSerial::default();

    (
        BackendReceiver {
            receiver: backend_recv,
            processed_serial: backend_serial.clone(),
        },
        BackendHandle {
            sender: backend_send,
            processed_serial: backend_serial,
            next_serial: Default::default(),
            sent: Default::default(),
        },
        FrontendReceiver {
            receiver: frontend_recv,
            processed_serial: frontend_serial.clone(),
        },
        FrontendHandle {
            sender: frontend_send,
            processed_serial: frontend_serial,
            next_serial: Default::default(),
        },
    )
}

#[derive(Debug)]
pub struct BackendReceiver {
    #[cfg(debug_assertions)]
    receiver: Receiver<(MessageToBackend, Option<Serial>)>,
    #[cfg(not(debug_assertions))]
    receiver: UnboundedReceiver<(MessageToBackend, Option<Serial>)>,
    processed_serial: AtomicSetSerial,
}

impl BackendReceiver {
    pub async fn recv(&mut self) -> Option<MessageToBackend> {
        let (message, serial) = self.receiver.recv().await?;
        if let Some(serial) = serial {
            self.processed_serial.set(serial);
        }
        Some(message)
    }
}

#[derive(Debug)]
pub struct FrontendReceiver {
    receiver: UnboundedReceiver<(MessageToFrontend, Option<Serial>)>,
    processed_serial: AtomicSetSerial,
}

impl FrontendReceiver {
    pub async fn recv(&mut self) -> Option<MessageToFrontend> {
        let (message, serial) = self.receiver.recv().await?;
        if let Some(serial) = serial {
            self.processed_serial.set(serial);
        }
        Some(message)
    }

    /// The next update if one is already queued. Lets the window take a burst
    /// in one go instead of waking up once per message.
    pub fn try_recv(&mut self) -> Option<MessageToFrontend> {
        let (message, serial) = self.receiver.try_recv().ok()?;
        if let Some(serial) = serial {
            self.processed_serial.set(serial);
        }
        Some(message)
    }

    /// How many updates are waiting.
    pub fn backlog(&self) -> usize {
        self.receiver.len()
    }
}

#[derive(Clone, Debug)]
pub struct BackendHandle {
    #[cfg(debug_assertions)]
    sender: Sender<(MessageToBackend, Option<Serial>)>,
    #[cfg(not(debug_assertions))]
    sender: UnboundedSender<(MessageToBackend, Option<Serial>)>,
    #[allow(dead_code)]
    processed_serial: AtomicSetSerial,
    #[allow(dead_code)]
    next_serial: AtomicSerialProvider,
    /// How many messages have gone out, ever.
    ///
    /// The overlay reads it once a second and shows the difference. A screen
    /// that asks the master for something while nobody touches it is the
    /// signature of a request made from `render`, which runs per frame — and
    /// that is a bug you cannot see by looking at the window.
    sent: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl BackendHandle {
    /// Called from the GPUI thread, so it must not be an async send. In debug a
    /// full channel blocks that thread, which means a frozen window.
    pub fn send(&self, message: MessageToBackend) {
        self.sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        #[cfg(debug_assertions)]
        let _ = self.sender.blocking_send((message, None));
        #[cfg(not(debug_assertions))]
        let _ = self.sender.send((message, None));
    }

    /// Total messages sent since start.
    pub fn sent_count(&self) -> u64 {
        self.sent.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Clone, Debug)]
pub struct FrontendHandle {
    sender: UnboundedSender<(MessageToFrontend, Option<Serial>)>,
    #[allow(dead_code)]
    processed_serial: AtomicSetSerial,
    #[allow(dead_code)]
    next_serial: AtomicSerialProvider,
}

impl FrontendHandle {
    /// Never blocks the runtime and never drops: a sync reports every file and
    /// a game can print thousands of lines in a second, and the update that
    /// says it exited comes right after them.
    pub fn send(&self, message: MessageToFrontend) {
        let _ = self.sender.send((message, None));
    }
}
