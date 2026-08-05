//! Версия запущенной сборки — мелкой строкой в углу.
//!
//! Раньше версию показывал только блок обновления, да и то чужую: ту, что
//! доступна к установке. Свою узнать было неоткуда, кроме файла `version` в
//! каталоге данных, а без неё непонятно, дошло ли обновление.

use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement};

/// Берётся из Cargo при компиляции, поэтому всегда соответствует бинарнику —
/// в отличие от файла на диске, который пишет bootstrapper.
pub fn version_badge() -> AnyElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(11.))
        .text_color(rgb(TEXT_MUTED))
        .child(format!("v{}", env!("CARGO_PKG_VERSION")))
        .into_any_element()
}
