//! IPC-канал между frontend (GPUI, главный поток) и backend (tokio).
//!
//! По образцу side-assist/"Пандоры": две пары mpsc-каналов, клонируемые отправители
//! и отдельные приёмники. UI остаётся синхронным (GPUI требует главный поток),
//! а вся сетевая/файловая работа живёт в tokio.

pub mod handle;
pub mod message;
pub mod modal_action;
pub mod quit;
pub mod serial;

pub use handle::{create_pair, BackendHandle, BackendReceiver, FrontendHandle, FrontendReceiver};
pub use message::{
    BuildState, CatalogHitInfo, ClientSettingsState, GameLogLevel, LoginErrorKind,
    MessageToBackend, MessageToFrontend, ModProjectInfo, OptionalModInfo, ServerSkinPresetItem,
    SyncStage,
};
pub use modal_action::{ModalAction, ModalProgress};
pub use quit::{QuitCoordinator, QuitHandler};
pub use serial::{AtomicSerialProvider, AtomicSetSerial, Serial};
