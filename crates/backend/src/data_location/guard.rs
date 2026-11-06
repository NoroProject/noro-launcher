//! Who is using the data root. A move needs it to itself: a sync writing
//! into an instance while it is copied would leave the copy behind the
//! original, and a game reading it while it is renamed would crash.

use bridge::DataStatus;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub struct DataUse {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    users: usize,
    moving: Option<Arc<AtomicBool>>,
    /// As the window was last told. An interrupted move waits for the
    /// player's decision, and until then the old folder must not change under
    /// the partial copy.
    status: DataStatus,
}

/// Held for as long as a sync, a game or a repair works in the data root.
pub struct UseGuard(Arc<DataUse>);

/// Held by the move; its flag is the move's cancel switch.
pub struct MoveGuard {
    owner: Arc<DataUse>,
    pub cancel: Arc<AtomicBool>,
}

impl DataUse {
    /// `None` while a move runs or waits to be resolved.
    pub fn enter(self: &Arc<Self>) -> Option<UseGuard> {
        let mut state = self.state.lock();
        if state.moving.is_some() || matches!(state.status, DataStatus::Interrupted { .. }) {
            return None;
        }
        state.users += 1;
        Some(UseGuard(self.clone()))
    }

    /// `None` while anything else uses the data root, or another move runs.
    pub fn begin_move(self: &Arc<Self>) -> Option<MoveGuard> {
        let mut state = self.state.lock();
        if state.users > 0 || state.moving.is_some() {
            return None;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        state.moving = Some(cancel.clone());
        Some(MoveGuard {
            owner: self.clone(),
            cancel,
        })
    }

    pub fn cancel_move(&self) {
        if let Some(cancel) = &self.state.lock().moving {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_busy(&self) -> bool {
        let state = self.state.lock();
        state.users > 0 || state.moving.is_some()
    }

    pub fn is_moving(&self) -> bool {
        self.state.lock().moving.is_some()
    }

    pub fn status(&self) -> DataStatus {
        self.state.lock().status.clone()
    }

    pub fn set_status(&self, status: DataStatus) {
        self.state.lock().status = status;
    }
}

impl Drop for UseGuard {
    fn drop(&mut self) {
        self.0.state.lock().users -= 1;
    }
}

impl Drop for MoveGuard {
    fn drop(&mut self) {
        self.owner.state.lock().moving = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_move_waits_for_the_game_and_the_game_for_the_move() {
        let data = Arc::new(DataUse::default());
        let game = data.enter().unwrap();
        assert!(data.begin_move().is_none());
        drop(game);

        let moving = data.begin_move().unwrap();
        assert!(data.enter().is_none());
        data.cancel_move();
        assert!(moving.cancel.load(Ordering::Relaxed));
        drop(moving);
        assert!(data.enter().is_some());
    }

    #[test]
    fn an_interrupted_move_keeps_launches_out() {
        let data = Arc::new(DataUse::default());
        data.set_status(DataStatus::Interrupted {
            from: "a".into(),
            to: "b".into(),
        });
        assert!(data.enter().is_none());
        assert!(data.begin_move().is_some(), "resuming it is a move");
    }
}
