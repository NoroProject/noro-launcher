//! The skin preview loop runs two independent clocks: limb sway always moves at
//! the same speed, while the rotation stands still for as long as the figure is
//! being dragged.

use crate::skin;
use crate::state::{LauncherUI, Page};
use gpui::Context;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Frame period. A whole frame — rasterise plus the BGRA swap — costs about
/// 4 ms, so 60 fps fits with room to spare; at 30 the rotation visibly stutters.
const FRAME: Duration = Duration::from_millis(16);
const SWAY_PERIOD_MS: f32 = 2400.0;
/// A full turn takes roughly six seconds.
const YAW_DEG_PER_SEC: f32 = 60.0;

pub(crate) struct FrameJob {
    skin: Vec<u8>,
    cape: Option<Vec<u8>>,
    yaw: f64,
    sway: f64,
}

impl LauncherUI {
    /// Advances the clocks. `None` means there is nothing to render.
    fn next_frame_job(&mut self, elapsed: Duration) -> Option<FrameJob> {
        let skin = self.skin_bytes.clone()?;
        if self.page != Page::Profile {
            return None;
        }
        let dt = elapsed.as_secs_f32().min(0.25); // don't jump after a long pause
        self.skin_sway = (self.skin_sway + dt * 1000.0 / SWAY_PERIOD_MS).fract();
        if !self.skin_dragging {
            self.skin_yaw = (self.skin_yaw + dt * YAW_DEG_PER_SEC).rem_euclid(360.0);
        }
        Some(FrameJob {
            skin,
            cape: self.cape_bytes.clone(),
            yaw: self.skin_yaw as f64,
            sway: self.skin_sway as f64,
        })
    }

    /// The loop picks the new angle up on its next frame; nothing to redraw
    /// here.
    pub fn rotate_skin(&mut self, degrees: f32, cx: &mut Context<Self>) {
        self.skin_yaw = (self.skin_yaw + degrees).rem_euclid(360.0);
        cx.notify();
    }

    pub(crate) fn start_skin_animation(&mut self, cx: &mut Context<Self>) {
        if self.skin_anim_running {
            return;
        }
        self.skin_anim_running = true;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let mut last = Instant::now();
            loop {
                let started = Instant::now();
                let elapsed = last.elapsed();
                last = started;
                let Ok(job) = this.update(cx, |state, _| state.next_frame_job(elapsed)) else {
                    break; // window closed
                };
                let Some(job) = job else {
                    // Profile closed or no skin: stop the loop rather than
                    // leave it waking up with nothing to draw.
                    let _ = this.update(cx, |state, _| state.skin_anim_running = false);
                    break;
                };

                let frame = executor
                    .spawn(async move {
                        skin::render_view(&job.skin, job.cape.as_deref(), job.yaw, job.sway)
                    })
                    .await;

                let alive = this.update(cx, |state, cx| {
                    // Frame didn't render: keep the previous one, or it blinks.
                    if let Some(frame) = frame {
                        // Hand the old frame back to GPUI. It caches every
                        // RenderImage in the sprite atlas by id and never evicts
                        // one on its own, so dropping our Arc frees the pixels
                        // but leaves the texture — half a megabyte every 16 ms.
                        //
                        // Unless a saved preset kept this exact frame: then the
                        // texture is still on screen and evicting it just makes
                        // GPUI upload it again on the next paint.
                        if let Some(stale) = state.skin_preview.replace(frame) {
                            if Arc::strong_count(&stale) == 1 {
                                cx.drop_image(stale, None);
                            }
                        }
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
                // Sleep for what's left of the period. Waiting a whole FRAME on
                // top of the render would put the real rate below the declared
                // one.
                executor
                    .timer(FRAME.saturating_sub(started.elapsed()))
                    .await;
            }
        })
        .detach();
    }
}
