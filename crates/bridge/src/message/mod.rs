//! Everything that crosses the frontend ↔ backend boundary.

use crate::modal_action::ModalAction;
use schema::{LauncherVersion, NewsItem, NotifLevel, ServerEntry, UserProfile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

mod data_location;
mod to_backend;
mod to_frontend;
mod types;
mod views;

pub use data_location::*;

pub use to_backend::MessageToBackend;
pub use to_frontend::MessageToFrontend;
pub use types::*;
pub use views::*;
