//! Кто сейчас в игре, поимённо, по каждому игровому серверу.
//!
//! Живёт в памяти, а не в базе: это снимок настоящего момента, переживать
//! перезапуск мастера ему незачем — состав приедет со следующим heartbeat.
//!
//! **События только ускоряют, истину задаёт heartbeat.** `join`/`leave`
//! приходят по каналу, который рвётся, поэтому раз в интервал агент присылает
//! полный список и [`Roster::reconcile`] заменяет им накопленное. Без этого
//! после каждого обрыва в списке оставались бы призраки.

use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// Состав одного игрового сервера.
///
/// Скрытые ванишем лежат отдельно от видимых: публичное число считается по
/// первым, а сессии и права — по обоим. Слить их в одно множество значит либо
/// показать скрытого модератора в списке, либо потерять его часы дежурства.
#[derive(Default, Clone, Debug)]
pub struct ServerRoster {
    pub visible: HashSet<Uuid>,
    pub vanished: HashSet<Uuid>,
}

impl ServerRoster {
    /// Все, кто на сервере, включая скрытых.
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
    /// Ключ — `game_servers.id`: состав у каждого сервера свой, даже если
    /// сборка одна на всех.
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

    /// Заменить состав тем, что прислал heartbeat.
    pub fn reconcile(&self, game_server_id: Uuid, visible: Vec<Uuid>, vanished: Vec<Uuid>) {
        self.servers.insert(
            game_server_id,
            ServerRoster {
                visible: visible.into_iter().collect(),
                vanished: vanished.into_iter().collect(),
            },
        );
    }

    /// Сервер отключился — состав больше ничем не подтверждён.
    pub fn clear(&self, game_server_id: Uuid) {
        self.servers.remove(&game_server_id);
    }

    pub fn of(&self, game_server_id: Uuid) -> ServerRoster {
        self.servers
            .get(&game_server_id)
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Где сидит игрок прямо сейчас. Нужно действиям из админки: кикать и
    /// писать в личку умеет только тот сервер, на котором игрок есть.
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
        assert!(snapshot.visible.is_empty(), "скрытый не остаётся видимым");
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
            "потерянный leave не оставляет призрака"
        );
        assert!(snapshot.contains(real));
    }
}
