//! The channel between the frontend (GPUI, main thread) and the backend (tokio).
//!
//! GPUI owns the main thread and stays synchronous, so anything network- or
//! disk-shaped runs on the tokio side and the two talk over a pair of mpsc
//! channels.

pub mod handle;
pub mod message;
pub mod modal_action;
pub mod quit;
pub mod serial;

pub use handle::{create_pair, BackendHandle, BackendReceiver, FrontendHandle, FrontendReceiver};
pub use message::{
    BuildState, CatalogHitInfo, ClientSettingsState, ContentVersionInfo, DmMessageView,
    DmThreadOpen, DmThreadView, GameLogLevel, GameLogLine, LoginErrorKind, MessageToBackend,
    MessageToFrontend, ModProjectInfo, OptionalModInfo, PunishmentView, RuleView, SanctionView,
    ServerSkinPresetItem, SyncStage, TicketMessageView, TicketView,
};
pub use modal_action::{ModalAction, ModalProgress};
pub use quit::{QuitCoordinator, QuitHandler};
pub use serial::{AtomicSerialProvider, AtomicSetSerial, Serial};
