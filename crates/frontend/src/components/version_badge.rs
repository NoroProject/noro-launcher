use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement};

/// Baked in by Cargo, so it always matches the running binary — unlike the
/// `version` file on disk, which the bootstrapper writes.
pub fn version_badge() -> AnyElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(11.))
        .text_color(rgb(TEXT_MUTED))
        .child(format!("v{}", env!("CARGO_PKG_VERSION")))
        .into_any_element()
}
