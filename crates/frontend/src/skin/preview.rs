//! GPUI-facing helper: один кадр превью в готовом для отрисовки виде.
//!
//! Кадры отдаются как `RenderImage`, а не как PNG-`Image`. `Image` декодируется
//! асинхронно через asset-кеш, поэтому при первом показе кадра рисовать нечего —
//! отсюда моргание. `RenderImage` резолвится синхронно и мимо кеша, так что
//! кадр появляется сразу и не оседает в памяти навсегда.

use super::{render_rgba, View};
use gpui::RenderImage;
use image::Frame;
use std::sync::Arc;

/// Preview size in logical px — matches the card in the profile page.
pub const PREVIEW_W: u32 = 280;
pub const PREVIEW_H: u32 = 340;
/// High performance scaling factor for silky smooth 60 FPS animation.
const SUPERSAMPLE_W: u32 = 350;
const SUPERSAMPLE_H: u32 = 425;

/// Slight downward tilt, so the figure is seen a bit from above.
const IDLE_PITCH: f64 = 6.0;

/// Отрисовать фигуру при заданном повороте и фазе покачивания.
/// `sway` — прогресс цикла конечностей в `[0, 1)`; с `yaw` он не связан, чтобы
/// вращение мышью не разгоняло и не отматывало анимацию рук и ног.
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
    // GPUI грузит текстуры как BGRA.
    for pixel in canvas.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Some(Arc::new(RenderImage::new(vec![Frame::new(canvas)])))
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
