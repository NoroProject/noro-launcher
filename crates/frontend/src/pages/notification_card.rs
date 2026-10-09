//! One notification in the panel: its icon, its text in the player's language, and
//! what clicking it does.

use super::common::Cx;
use crate::icons::ic;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use schema::notifications::{Level, Notification};

pub(super) fn card(n: &Notification, cx: &mut Cx) -> AnyElement {
    let id = n.id;
    let unread = !n.read;
    let (icon, colour) = look(n.level);
    let title = translated(n.title_key.as_deref(), &n.title, &n.args);
    let body = translated(n.body_key.as_deref(), &n.body, &n.args);
    let link = n.link.clone();

    div()
        .id(gpui::SharedString::from(format!("notif-{id}")))
        .p(px(12.))
        .rounded(px(R_SM))
        .flex()
        .gap(px(12.))
        .cursor_pointer()
        .bg(if unread {
            rgba((ACCENT << 8) | 0x10)
        } else {
            rgba(0xffffff05)
        })
        .hover(|d| d.bg(rgba(0xffffff12)))
        .child(div().pt(px(2.)).child(ic(icon, 16., colour)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(if unread {
                            FontWeight::BOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(title),
                )
                .when(!body.is_empty(), |d| {
                    d.child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(body),
                    )
                })
                .when(n.repeat_count > 1, |d| {
                    d.child(
                        div()
                            .text_size(px(10.))
                            .text_color(rgb(TEXT_MUTED))
                            .child(format!("×{}", n.repeat_count)),
                    )
                }),
        )
        // Clicking marks it read and, when the card leads somewhere, opens that
        // in a browser: the destination is a page on the site, and the launcher
        // has no second copy of it.
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            if unread {
                this.backend
                    .send(MessageToBackend::MarkNotificationRead { id });
                if let Some(n) = this.notifications.iter_mut().find(|n| n.id == id) {
                    n.read = true;
                }
            }
            if let Some(href) = &link {
                // The path in a card is a website page, and the launcher has no website
                // address: only the master is baked in. The master redirects to the site,
                // as it does for signing in through the browser. The path used to be glued
                // to the master address, and the link led to the API instead of the page.
                let url = if href.starts_with("http") {
                    href.clone()
                } else {
                    format!(
                        "{}/go?to={}",
                        this.config.master_url.trim_end_matches('/'),
                        urlencoding::encode(href)
                    )
                };
                let _ = open::that_detached(url);
            }
            cx.notify();
        }))
        .into_any_element()
}

fn look(level: Level) -> (&'static str, u32) {
    match level {
        Level::Urgent => ("circle-alert", ERROR),
        Level::Important => ("triangle-alert", WARNING),
        Level::Normal => ("info", BLUE),
    }
}

/// Prefer our own catalogue over the master's finished string.
///
/// The master translates with the locale it runs in. For the site that is
/// right — it reads the same catalogue from the master — but the launcher has
/// its own, in the language the player chose here, and showing them Russian
/// because the server runs in Russian is not a translation.
fn translated(
    key: Option<&str>,
    fallback: &str,
    args: &std::collections::BTreeMap<String, String>,
) -> String {
    let Some(key) = key else {
        return fallback.to_string();
    };
    let translated = if args.is_empty() {
        t(key)
    } else {
        let mut fluent = i18n::FluentArgs::new();
        for (k, v) in args {
            fluent.set(k.clone(), v.clone());
        }
        i18n::t_args(key, &fluent)
    };
    // `t` hands back the key itself when it knows nothing about it. The
    // master's own rendering is a better answer than a raw key.
    if translated == key {
        fallback.to_string()
    } else {
        translated
    }
}
