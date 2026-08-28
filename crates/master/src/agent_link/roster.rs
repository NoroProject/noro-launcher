//! Who is in game right now, by name, per game server.
//!
//! In memory rather than in the database: it's a snapshot of the present, and
//! the roster arrives again with the next heartbeat anyway.
//!
//! Events only make it faster; the heartbeat is the truth. `join`/`leave`
//! travel over a channel that drops, so the agent periodically sends the full
//! list and [`Roster::reconcile`] replaces what accumulated. Otherwise every
//! disconnect would leave ghosts behind.

use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// One game server's roster.
///
/// Vanished players are kept apart from visible ones: the public count uses
/// only the visible set, while sessions and permissions use both. Merging them
/// would either expose a vanished moderator or lose their on-duty hours.
#[derive(Default, Clone, Debug)]
pub struct ServerRoster {
    pub visible: HashSet<Uuid>,
    pub vanished: HashSet<Uuid>,
}

impl ServerRoster {
    /// Everyone on the server, vanished included.
    pub fn everyone(&self) -> impl Iterator<Item = &Uuid> {
        self.visible.iter().chain(self.vanished.iter())
    }

    pub fn contains(&self, uuid: Uuid) -> bool {
        self.visible.contains(&uuid) || self.vanished.contains(&uuid)
    }

    fn insert(&mut self, uuid: Uuid, vanished: bool) {
        if vanished {
            self.visible.remove(&uuid);
            self.vanished.insert(uuid);
        } else {
            self.vanished.remove(&uuid);
            self.visible.insert(uuid);
        }
    }

    fn remove(&mut self, uuid: Uuid) {
        self.visible.remove(&uuid);
        self.vanished.remove(&uuid);
    }
}

#[derive(Clone, Default)]
pub struct Roster {
    /// Keyed by `game_servers.id` — each server has its own roster even when
    /// several share a build.
    servers: Arc<DashMap<Uuid, ServerRoster>>,
}

impl Roster {
    pub fn join(&self, game_server_id: Uuid, uuid: Uuid, vanished: bool) {
        self.servers
            .entry(game_server_id)
            .or_default()
            .insert(uuid, vanished);
    }

    pub fn leave(&self, game_server_id: Uuid, uuid: Uuid) {
        if let Some(mut roster) = self.servers.get_mut(&game_server_id) {
            roster.remove(uuid);
        }
    }

    /// Replace the roster with what the heartbeat reported.
    pub fn reconcile(&self, game_server_id: Uuid, visible: Vec<Uuid>, vanished: Vec<Uuid>) {
        self.servers.insert(
            game_server_id,
            ServerRoster {
                visible: visible.into_iter().collect(),
                vanished: vanished.into_iter().collect(),
            },
        );
    }

    /// Server disconnected — nothing confirms the roster any more.
    pub fn clear(&self, game_server_id: Uuid) {
        self.servers.remove(&game_server_id);
    }

    pub fn of(&self, game_server_id: Uuid) -> ServerRoster {
        self.servers
            .get(&game_server_id)
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Which server the player is on right now. Admin actions need it — only
    /// the server holding the player can kick them or message them.
    pub fn locate(&self, uuid: Uuid) -> Option<Uuid> {
        self.servers
            .iter()
            .find(|entry| entry.contains(uuid))
            .map(|entry| *entry.key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanished_moves_between_lists() {
        let roster = Roster::default();
        let server = Uuid::new_v4();
        let player = Uuid::new_v4();

        roster.join(server, player, false);
        assert!(roster.of(server).visible.contains(&player));

        roster.join(server, player, true);
        let snapshot = roster.of(server);
        assert!(snapshot.visible.is_empty(), "vanished must not stay visible");
        assert!(snapshot.vanished.contains(&player));
        assert_eq!(roster.locate(player), Some(server));
    }

    #[test]
    fn reconcile_drops_ghosts() {
        let roster = Roster::default();
        let server = Uuid::new_v4();
        let ghost = Uuid::new_v4();
        let real = Uuid::new_v4();

        roster.join(server, ghost, false);
        roster.reconcile(server, vec![real], vec![]);

        let snapshot = roster.of(server);
        assert!(
            !snapshot.contains(ghost),
            "a lost leave must not leave a ghost"
        );
        assert!(snapshot.contains(real));
    }
}
