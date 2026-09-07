//! GPUI-facing helper: one preview frame, ready to draw.
//!
//! Frames come back as `RenderImage` rather than a PNG `Image`. `Image` decodes
//! asynchronously through the asset cache, so the first time a frame is shown
//! there is nothing to draw yet — that's the blink. `RenderImage` resolves
//! synchronously and past the cache, so the frame is there at once and doesn't
//! stay in memory forever.

use super::{render_rgba, View};
use gpui::RenderImage;
use image::Frame;
use std::sync::Arc;

/// Preview size in logical px — matches the card in the profile page.
pub const PREVIEW_W: u32 = 280;
pub const PREVIEW_H: u32 = 340;
/// Rendered larger than the preview and scaled down — cheap antialiasing.
const SUPERSAMPLE_W: u32 = 350;
const SUPERSAMPLE_H: u32 = 425;

/// Slight downward tilt, so the figure is seen a bit from above.
const IDLE_PITCH: f64 = 6.0;

/// `sway` is the limb cycle's progress in `[0, 1)`. It is deliberately
/// unrelated to `yaw`, so dragging the figure with the mouse doesn't speed up
/// or rewind the arms and legs.
pub fn render_view(
    skin_png: &[u8],
    cape_png: Option<&[u8]>,
    yaw: f64,
    sway: f64,
) -> Option<Arc<RenderImage>> {
    let view = View {
        yaw,
        pitch: IDLE_PITCH,
        sway,
        ..View::default()
    };
    let mut canvas = render_rgba(skin_png, cape_png, SUPERSAMPLE_W, SUPERSAMPLE_H, &view)?;
    // GPUI uploads textures as BGRA.
    for pixel in canvas.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Some(Arc::new(RenderImage::new(vec![Frame::new(canvas)])))
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
