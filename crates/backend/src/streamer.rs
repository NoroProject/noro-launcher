//! Streamer mode: what the window, Discord and the in-game panel keep to
//! themselves while the player is on air.
//!
//! Two ways in. The player's own switch (settings or the shortcut) is saved,
//! so restarting the launcher mid-stream reveals nothing. The watcher turns the
//! mode on while a streaming app runs, and acts only when that changes — OBS
//! starting or closing — so a player who switched it off by hand during a
//! stream is not overruled every few seconds.

use crate::backend::Ctx;
use bridge::MessageToFrontend;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

mod processes;

/// How often the process list is read. Starting OBS takes longer than this,
/// and the list is read on a blocking thread, never in a frame.
const CHECK_EVERY: Duration = Duration::from_secs(15);

#[derive(Debug, Default)]
pub struct Streamer {
    on: AtomicBool,
    /// On because a streaming app runs, not because the player asked. Only
    /// then does the app closing turn it back off.
    automatic: AtomicBool,
}

impl Streamer {
    pub fn new(on: bool) -> Self {
        Self {
            on: AtomicBool::new(on),
            automatic: AtomicBool::new(false),
        }
    }

    pub fn on(&self) -> bool {
        self.on.load(Ordering::Relaxed)
    }

    fn automatic(&self) -> bool {
        self.automatic.load(Ordering::Relaxed)
    }
}

/// The player flipped the switch.
pub fn set_manual(ctx: &Ctx, on: bool) {
    ctx.config.update(|c| c.streamer_mode = on);
    apply(ctx, on, false);
}

/// The player stopped the watcher. A mode it switched on stays on: switching
/// the automation off is not a request to reveal anything mid-stream.
pub fn set_auto(ctx: &Ctx, enabled: bool) {
    ctx.config.update(|c| c.streamer_auto = enabled);
    if !enabled {
        ctx.streamer.automatic.store(false, Ordering::Relaxed);
    }
}

/// Discord's status follows both switches: the player's and this mode.
pub fn rpc_allowed(ctx: &Ctx) -> bool {
    ctx.config.get().discord_rpc && !ctx.streamer.on()
}

fn apply(ctx: &Ctx, on: bool, automatic: bool) {
    let was = ctx.streamer.on.swap(on, Ordering::Relaxed);
    ctx.streamer
        .automatic
        .store(on && automatic, Ordering::Relaxed);
    // The status names the network and the server, which is where a viewer
    // goes looking for the player. Off for the whole stream, not trimmed.
    ctx.rpc.set_enabled(rpc_allowed(ctx));
    ctx.send(MessageToFrontend::StreamerMode { on, automatic });
    if was != on {
        ctx.mod_link.streamer_changed(ctx);
    }
}

pub fn spawn_watcher(ctx: Ctx) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(CHECK_EVERY);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut seen = false;
        loop {
            tick.tick().await;
            if !ctx.config.get().streamer_auto {
                seen = false;
                continue;
            }
            let running = tokio::task::spawn_blocking(processes::streaming_app_running)
                .await
                .unwrap_or(false);
            if running == seen {
                continue;
            }
            seen = running;
            if running && !ctx.streamer.on() {
                apply(&ctx, true, true);
            } else if !running && ctx.streamer.automatic() {
                apply(&ctx, false, false);
            }
        }
    });
}
