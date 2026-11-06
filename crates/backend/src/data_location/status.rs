//! What the data root looks like when the launcher starts.

use super::journal::{Journal, Phase};
use crate::directories::LauncherDirectories;

#[derive(Debug, PartialEq, Eq)]
pub enum Startup {
    Ready,
    /// A moved data root that is not there: a disconnected disk, most likely.
    Unavailable,
    /// A move died before its switch; the player decides what becomes of it.
    Interrupted(Journal),
    /// A move died after its switch, while deleting the old copy. Nothing to
    /// ask: the deleting just goes on.
    Cleanup(Journal),
}

pub fn startup(dirs: &LauncherDirectories) -> Startup {
    let current = dirs.data_root();
    match Journal::load(&dirs.move_journal()).map(|j| j.settled(&current)) {
        Some(journal) if journal.phase == Phase::Cleanup => Startup::Cleanup(journal),
        // The move started from somewhere the config no longer points at —
        // the player put the old path back by hand. Its journal means nothing.
        Some(journal) if journal.from != current => {
            Journal::clear(&dirs.move_journal());
            status_of(dirs)
        }
        Some(journal) => Startup::Interrupted(journal),
        None => status_of(dirs),
    }
}

fn status_of(dirs: &LauncherDirectories) -> Startup {
    if dirs.data_available() {
        Startup::Ready
    } else {
        Startup::Unavailable
    }
}
