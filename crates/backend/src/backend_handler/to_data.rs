// Over 150 lines: the requests about the game files folder and the start-up
// check, which share the steps of telling the window where things stand.
//! The game files folder: its size, moving it, and what start-up finds.
//! The work itself is in [`crate::data_location`]; this is the wiring.

use super::*;
use crate::backend::Ctx;
use crate::data_location::{self as data, MoveError, MoveRequest, Refusal, Startup};
use bridge::{DataLocationView, DataMoveOutcome, DataStatus};
use std::path::{Path, PathBuf};

impl BackendState {
    pub(super) async fn on_data_request(&mut self, msg: MessageToBackend) {
        use MessageToBackend as M;
        let ctx = self.ctx.clone();
        match msg {
            M::RequestDataLocation => blocking(move || {
                recheck(&ctx);
                send_location(&ctx, true);
            }),
            M::PlanDataMove { target } => blocking(move || plan(&ctx, &target)),
            M::StartDataMove { target } => start(ctx, target),
            M::CancelDataMove => ctx.data_use.cancel_move(),
            M::ResolveDataMove { resume } => resolve(ctx, resume),
            M::ResetDataLocation => blocking(move || reset(&ctx)),
            M::OpenDataFolder => {
                let _ = open::that(ctx.dirs.data_root());
            }
            _ => {}
        }
    }

    /// Why nothing may start in the data root right now, as the key the
    /// window shows, or `None` when it may.
    pub(super) fn data_blocker(&self) -> Option<&'static str> {
        if !self.ctx.dirs.data_available() {
            return Some("data-unavailable-launch");
        }
        let status = self.ctx.data_use.status();
        (self.ctx.data_use.is_moving() || matches!(status, DataStatus::Interrupted { .. }))
            .then_some("data-move-launch-blocked")
    }
}

/// Run at start, once the window can be told: a disconnected disk or a move
/// that died gets reported instead of downloads starting over in its place.
pub(crate) fn data_startup(ctx: &Ctx) {
    let status = match data::startup(&ctx.dirs) {
        Startup::Ready => DataStatus::Ready,
        Startup::Unavailable => {
            tracing::warn!(path = %ctx.dirs.data_root().display(), "data root not available");
            DataStatus::Unavailable
        }
        Startup::Interrupted(journal) => {
            tracing::warn!(?journal, "found an interrupted data move");
            DataStatus::Interrupted {
                from: journal.from,
                to: journal.to,
            }
        }
        Startup::Cleanup(journal) => {
            let ctx = ctx.clone();
            blocking(move || {
                let dirs = &ctx.dirs;
                let left = data::finish_cleanup(
                    &journal,
                    dirs.root(),
                    &dirs.move_journal(),
                    &dirs.data_root(),
                );
                if let Some(left) = left {
                    notify_leftover(&ctx, &left);
                }
            });
            DataStatus::Ready
        }
    };
    if status == DataStatus::Ready {
        if let Err(e) = ctx.dirs.ensure_data() {
            tracing::error!(error = %e, "data folders not created");
        }
    }
    ctx.data_use.set_status(status);
    send_location(ctx, false);
}

fn blocking(f: impl FnOnce() + Send + 'static) {
    drop(tokio::task::spawn_blocking(f));
}

/// `count` walks the whole folder for its size — only when the settings page
/// is looking, never at start.
fn send_location(ctx: &Ctx, count: bool) {
    let dirs = &ctx.dirs;
    let path = dirs.data_root();
    let status = ctx.data_use.status();
    let there = status != DataStatus::Unavailable;
    let view = DataLocationView {
        size_bytes: (count && there)
            .then(|| data::scan(&path).ok().map(|t| t.bytes))
            .flatten(),
        free_bytes: there
            .then(|| crate::disk_space::available(&path).ok())
            .flatten(),
        default_path: dirs.root().clone(),
        is_default: dirs.data_root_is_default(),
        path,
        status,
    };
    ctx.send(MessageToFrontend::DataLocation { view });
}

/// The checks shared by planning and starting. `None` from a check that
/// failed has already been reported.
fn checked_plan(ctx: &Ctx, target: &Path) -> Option<data::Plan> {
    let refusal = match ctx.data_use.status() {
        DataStatus::Unavailable => Some(Refusal::new("data-move-refused-unavailable")),
        DataStatus::Interrupted { .. } => Some(Refusal::new("data-move-refused-pending")),
        DataStatus::Ready => None,
    };
    let result = match refusal {
        Some(r) => Err(r),
        None => data::plan(
            ctx.dirs.root(),
            &ctx.dirs.data_root(),
            target,
            &data::RealDisk,
        ),
    };
    result.map_err(|r| refuse(ctx, r)).ok()
}

fn plan(ctx: &Ctx, target: &Path) {
    if ctx.data_use.is_busy() {
        return refuse(ctx, Refusal::new("data-move-refused-busy"));
    }
    if let Some(plan) = checked_plan(ctx, target) {
        ctx.send(MessageToFrontend::DataMovePlanned { plan });
    }
}

fn refuse(ctx: &Ctx, r: Refusal) {
    ctx.send(MessageToFrontend::DataMoveRefused {
        key: r.key.into(),
        args: r.args,
    });
}

fn start(ctx: Ctx, target: PathBuf) {
    let Some(guard) = ctx.data_use.begin_move() else {
        return refuse(&ctx, Refusal::new("data-move-refused-busy"));
    };
    blocking(move || {
        let Some(plan) = checked_plan(&ctx, &target) else {
            return;
        };
        let req = MoveRequest {
            service_root: ctx.dirs.root().clone(),
            journal: ctx.dirs.move_journal(),
            from: ctx.dirs.data_root(),
            to: plan.target,
            same_volume: plan.same_volume,
        };
        carry_out(&ctx, &req, guard);
    });
}

fn resolve(ctx: Ctx, resume: bool) {
    let Some(guard) = ctx.data_use.begin_move() else {
        return refuse(&ctx, Refusal::new("data-move-refused-busy"));
    };
    blocking(move || {
        let dirs = &ctx.dirs;
        let Some(journal) = data::Journal::load(&dirs.move_journal()) else {
            return settle(&ctx);
        };
        if resume {
            let req = MoveRequest {
                service_root: dirs.root().clone(),
                journal: dirs.move_journal(),
                same_volume: journal.phase == data::Phase::Renaming,
                from: journal.from,
                to: journal.to,
            };
            return carry_out(&ctx, &req, guard);
        }
        if data::rollback(
            &journal,
            dirs.root(),
            &dirs.move_journal(),
            &dirs.data_root(),
        ) {
            drop(guard);
            settle(&ctx);
            ctx.send(MessageToFrontend::DataMoveFinished {
                outcome: DataMoveOutcome::Cancelled,
            });
        } else {
            drop(guard);
            refuse(&ctx, Refusal::new("data-move-error-access").at(&journal.to));
        }
    });
}

fn carry_out(ctx: &Ctx, req: &MoveRequest, guard: data::MoveGuard) {
    let progress_ctx = ctx.clone();
    let mut sink =
        move |progress| progress_ctx.send(MessageToFrontend::DataMoveProgress { progress });
    let mut meter = data::Meter::new(&guard.cancel, &mut sink);
    let result = data::run(req, &mut meter, &mut |to| switch(ctx, to));
    let outcome = match result {
        Ok(done) => {
            if let Err(e) = ctx.dirs.ensure_data() {
                tracing::error!(error = %e, "data folders not created");
            }
            DataMoveOutcome::Done {
                path: req.to.clone(),
                leftover: done.leftover,
            }
        }
        Err(MoveError::Cancelled) => DataMoveOutcome::Cancelled,
        Err(MoveError::Failed(r)) => {
            tracing::warn!(key = r.key, args = ?r.args, "data move failed");
            DataMoveOutcome::Failed {
                path: req.from.clone(),
                key: r.key.into(),
                args: r.args,
            }
        }
    };
    drop(guard);
    settle(ctx);
    ctx.send(MessageToFrontend::DataMoveFinished { outcome });
}

/// The one step that makes a move real: the config first, and the paths
/// every task uses only once the config holds them.
fn switch(ctx: &Ctx, to: &Path) -> std::io::Result<()> {
    let data_dir = (to != ctx.dirs.root().as_path()).then(|| to.to_path_buf());
    ctx.config.try_update(|c| c.data_dir = data_dir.clone())?;
    ctx.dirs.set_data_root(data_dir.as_deref());
    Ok(())
}

/// After a move ends, however it ended: the journal on disk says whether
/// something is still left to decide.
fn settle(ctx: &Ctx) {
    let status = match data::startup(&ctx.dirs) {
        Startup::Interrupted(j) => DataStatus::Interrupted {
            from: j.from,
            to: j.to,
        },
        Startup::Unavailable => DataStatus::Unavailable,
        Startup::Ready | Startup::Cleanup(_) => DataStatus::Ready,
    };
    ctx.data_use.set_status(status);
    send_location(ctx, true);
}

/// The disk is gone and the player stops waiting for it. Nothing on it is
/// touched; the next launch downloads into the standard folder.
fn reset(ctx: &Ctx) {
    if ctx.data_use.status() != DataStatus::Unavailable {
        return;
    }
    if let Err(e) = ctx.config.try_update(|c| c.data_dir = None) {
        tracing::error!(error = %e, "could not go back to the standard folder");
        return refuse(ctx, Refusal::new("data-move-error-config"));
    }
    ctx.dirs.set_data_root(None);
    if let Err(e) = ctx.dirs.ensure_data() {
        tracing::error!(error = %e, "data folders not created");
    }
    ctx.data_use.set_status(DataStatus::Ready);
    send_location(ctx, false);
}

/// A disk can come and go while the launcher runs: plugged back in after the
/// start-up warning, or pulled out in the middle of a session.
fn recheck(ctx: &Ctx) {
    let available = ctx.dirs.data_available();
    match ctx.data_use.status() {
        DataStatus::Unavailable if available => {
            if let Err(e) = ctx.dirs.ensure_data() {
                tracing::error!(error = %e, "data folders not created");
            }
            ctx.data_use.set_status(DataStatus::Ready);
        }
        DataStatus::Ready if !available => ctx.data_use.set_status(DataStatus::Unavailable),
        _ => {}
    }
}

fn notify_leftover(ctx: &Ctx, left: &Path) {
    ctx.send(MessageToFrontend::AddNotification {
        key: "data-move-done-leftover".into(),
        args: [("path".to_string(), left.display().to_string())].into(),
        level: schema::NotifLevel::Warning,
    });
}
