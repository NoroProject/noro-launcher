//! UI-компоненты лаунчера. Один компонент = один файл (≤150 строк).
// Часть компонентов — библиотечные, ещё не везде задействованы.
#![allow(unused_imports)]

mod atom_art;
mod badge;
mod button;
mod checkbox;
mod cta_button;
mod icon_button;
mod mascot;
mod mod_toggle;
mod pixel_title;
mod progress;
mod version_badge;
mod window_chrome;

pub use atom_art::{atom_art, tiny_atom_logo};
pub use badge::badge;
pub use button::btn;
pub use checkbox::checkbox_row;
pub use cta_button::cta_button;
pub use icon_button::{icon_btn, stepper_btn};
pub use mascot::{mascot, Mood};
pub use mod_toggle::mod_toggle;
pub use pixel_title::{pixel_label, pixel_title};
pub use progress::progress_bar;
pub use version_badge::version_badge;
pub use window_chrome::window_chrome;
