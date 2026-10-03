//! Backend messages about notifications, punishments, rules, tickets and messages.

use super::*;

impl LauncherUI {
    /// Only ever gets the variants `on_message` routes here.
    pub(super) fn on_account_message(&mut self, msg: MessageToFrontend) {
        match msg {
            MessageToFrontend::NotificationFeed {
                items,
                total,
                offset,
                unread,
            } => {
                // Offset 0 is a refresh, anything else a further page. Appending
                // both ways would double the feed every time the panel reopens.
                if offset == 0 {
                    self.notifications = items;
                } else {
                    self.notifications.extend(items);
                }
                self.notifications_total = total;
                self.unread = unread;
                self.notifications_loading = false;
            }
            MessageToFrontend::NotificationArrived {
                notification,
                unread,
                ..
            } => {
                // Newest first, and a repeat of something already listed
                // replaces it: the master collapses repeats into one row with a
                // counter, and keeping the old copy would show both.
                self.notifications.retain(|n| n.id != notification.id);
                self.notifications.insert(0, *notification);
                self.unread = unread;
            }
            MessageToFrontend::UnreadChanged { unread } => {
                self.unread = unread;
            }

            MessageToFrontend::PunishmentsLoaded { items } => {
                self.punishments = items;
                self.account_loaded.insert("punishments");
            }
            MessageToFrontend::RulesLoaded { items } => {
                self.rules = items;
                self.account_loaded.insert("rules");
            }
            MessageToFrontend::TicketsLoaded { items } => {
                self.tickets = items;
                self.account_loaded.insert("tickets");
            }
            MessageToFrontend::TicketLoaded { id, messages, .. } => {
                // Subject and status come from the list: the messages endpoint returns
                // only the messages, and putting empty strings here would erase the
                // title of the open ticket.
                let (subject, status) = self
                    .tickets
                    .iter()
                    .find(|t| t.id == id)
                    .map(|t| (t.subject.clone(), t.status.clone()))
                    .unwrap_or_default();
                self.ticket_open = Some((id, subject, status, messages));
                self.ticket_scroll.scroll_to_bottom();
                self.compose.clear();
            }
            MessageToFrontend::DmThreadsLoaded { items } => {
                self.dm_threads = items;
                self.dm_loaded = true;
            }
            MessageToFrontend::DmThreadLoaded { thread } => {
                self.dm_open = Some(*thread);
                self.dm_scroll.scroll_to_bottom();
                self.compose.clear();
            }
            MessageToFrontend::DmArrived { peer, message } => {
                // Only into the conversation that is open. The list of threads
                // is refetched instead: its previews and unread counts are the
                // master's arithmetic, not ours.
                if let Some(open) = self.dm_open.as_mut() {
                    if open.peer == peer {
                        open.messages.push(message);
                        self.dm_scroll.scroll_to_bottom();
                    }
                }
                self.backend.send(MessageToBackend::RequestDmThreads);
            }

            _ => {}
        }
    }
}
