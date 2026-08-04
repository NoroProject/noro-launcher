//! Парные ручки канала между frontend (GPUI, главный поток) и backend (tokio).
//!
//! По образцу "Пандоры"/side-assist: четыре сущности.
//! - [`BackendHandle`] — клонируемый отправитель команд (frontend/main → backend).
//! - [`BackendReceiver`] — приёмник команд (у backend).
//! - [`FrontendHandle`] — клонируемый отправитель обновлений (backend → frontend).
//! - [`FrontendReceiver`] — приёмник обновлений (у frontend).
//!
//! В debug каналы ограничены (ловит переполнение раньше), в release — unbounded.

#[cfg(debug_assertions)]
use tokio::sync::mpsc::{Receiver, Sender};
#[cfg(not(debug_assertions))]
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::message::{MessageToBackend, MessageToFrontend};
use crate::serial::{AtomicSerialProvider, AtomicSetSerial, Serial};

pub fn create_pair() -> (
    BackendReceiver,
    BackendHandle,
    FrontendReceiver,
    FrontendHandle,
) {
    #[cfg(debug_assertions)]
    let (frontend_send, frontend_recv) = tokio::sync::mpsc::channel(256);
    #[cfg(debug_assertions)]
    let (backend_send, backend_recv) = tokio::sync::mpsc::channel(256);

    #[cfg(not(debug_assertions))]
    let (frontend_send, frontend_recv) = tokio::sync::mpsc::unbounded_channel();
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
    #[cfg(debug_assertions)]
    receiver: Receiver<(MessageToFrontend, Option<Serial>)>,
    #[cfg(not(debug_assertions))]
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
}

impl BackendHandle {
    /// Отправить команду backend. Вызывается из синхронного GPUI-потока.
    pub fn send(&self, message: MessageToBackend) {
        #[cfg(debug_assertions)]
        let _ = self.sender.blocking_send((message, None));
        #[cfg(not(debug_assertions))]
        let _ = self.sender.send((message, None));
    }
}

#[derive(Clone, Debug)]
pub struct FrontendHandle {
    #[cfg(debug_assertions)]
    sender: Sender<(MessageToFrontend, Option<Serial>)>,
    #[cfg(not(debug_assertions))]
    sender: UnboundedSender<(MessageToFrontend, Option<Serial>)>,
    #[allow(dead_code)]
    processed_serial: AtomicSetSerial,
    #[allow(dead_code)]
    next_serial: AtomicSerialProvider,
}

impl FrontendHandle {
    /// Отправить обновление во frontend. Вызывается из async backend.
    pub fn send(&self, message: MessageToFrontend) {
        #[cfg(debug_assertions)]
        let _ = self.sender.try_send((message, None));
        #[cfg(not(debug_assertions))]
        let _ = self.sender.send((message, None));
    }
}
