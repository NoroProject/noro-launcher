//! The notification feed, and the system toast that goes with it.
//!
//! The feed itself lives on the master — it is the same one the site shows, and
//! a launcher keeping its own copy would disagree with the website about what
//! has been read. What is local is the toast: only this process knows whether
//! its window is even visible.
//!
//! The master decides which client raises that toast. It is the only party that
//! can see both an open browser tab and a running launcher, and two clients
//! each deciding for themselves means two identical toasts for one event.

use crate::backend::Ctx;
use crate::master_api::MasterApi;
use bridge::MessageToFrontend;
use uuid::Uuid;

const PAGE: u32 = 30;

fn api(ctx: &Ctx) -> Option<MasterApi> {
    MasterApi::new(
        ctx.http.clone(),
        &ctx.config.get().master_url,
        ctx.ws.token(),
    )
}

/// One page of the feed. `offset` of 0 is a refresh, anything else is "load
/// more" — the panel decides which, and the frontend gets told which it got.
pub fn request(ctx: &Ctx, offset: u32, unread_only: bool) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        let page = match api.notifications(offset, PAGE, unread_only).await {
            Ok(page) => page,
            Err(e) => {
                tracing::debug!(error = %e, "could not load the notification feed");
                return;
            }
        };
        // The unread count is a separate request on purpose: the page may be
        // filtered to unread only, in which case its length is not the count,
        // and a badge that disagrees with the list is worse than a late one.
        let unread = api.unread_count().await.unwrap_or(0);
        ctx.send(MessageToFrontend::NotificationFeed {
            items: page.items,
            total: page.total,
            offset,
            unread,
        });
    });
}

pub fn mark_read(ctx: &Ctx, id: Uuid) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        match api.mark_read(id).await {
            Ok(unread) => ctx.send(MessageToFrontend::UnreadChanged { unread }),
            Err(e) => tracing::debug!(error = %e, "could not mark the notification read"),
        }
    });
}

pub fn mark_all_read(ctx: &Ctx) {
    let ctx = ctx.clone();
    tokio::spawn(async move {
        let Some(api) = api(&ctx) else { return };
        if let Err(e) = api.mark_all_read().await {
            tracing::debug!(error = %e, "could not mark the feed read");
            return;
        }
        ctx.send(MessageToFrontend::UnreadChanged { unread: 0 });
        // The cards themselves carry a read flag, so the panel needs the page
        // again rather than a count it cannot apply to what it is drawing.
        request(&ctx, 0, false);
    });
}

/// A notification that arrived over the socket.
pub fn arrived(
    ctx: &Ctx,
    notification: schema::notifications::Notification,
    unread: i64,
    os_toast: bool,
) {
    if os_toast {
        toast(&notification);
    }
    ctx.send(MessageToFrontend::NotificationArrived {
        notification: Box::new(notification),
        unread,
        os_toast,
    });
}

/// Raise a system notification.
///
/// Best effort by design: a desktop with no notification daemon, a user who
/// switched them off at the OS level and a sandbox with no D-Bus all end up
/// here, and none of them is a reason to log an error — the card is in the
/// launcher's own panel either way.
fn toast(notification: &schema::notifications::Notification) {
    let summary = notification.title.clone();
    let body = notification.body.clone();
    std::thread::spawn(move || {
        let mut builder = notify_rust::Notification::new();
        builder.summary(&summary).appname("Noro");
        if !body.is_empty() {
            // Only the first lines fit in a system toast on every platform, and
            // the full text is a click away in the launcher.
            builder.body(&shorten(&body, 160));
        }
        if let Err(e) = builder.show() {
            tracing::debug!(error = %e, "the system would not show a notification");
        }
    });
}

/// Cut on a word boundary, so a toast does not end mid-word.
fn shorten(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let cut: String = text.chars().take(limit).collect();
    let end = cut.rfind(' ').unwrap_or(cut.len());
    format!("{}…", cut[..end].trim_end())
}

#[cfg(test)]
mod tests {
    use super::shorten;

    #[test]
    fn short_text_is_left_alone() {
        assert_eq!(shorten("hello", 160), "hello");
    }

    #[test]
    fn long_text_stops_at_a_word() {
        assert_eq!(shorten("one two three", 8), "one two…");
    }

    #[test]
    fn cyrillic_counts_characters_and_not_bytes() {
        // Cutting by bytes would split a two-byte character and panic.
        let text = "проверка длинной строки";
        assert!(shorten(text, 8).chars().count() <= 9);
    }
}
