//! How the log is shown: which columns, wrapping, text size. Kept in the
//! launcher's config, so a console set up once opens the same way again.

use crate::console_window::ConsoleWindow;
use crate::icons::ic;
use crate::theme::*;
use bridge::ConsoleSettings;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, Context, FontWeight, SharedString};
use i18n::t;

type Cx<'a> = Context<'a, ConsoleWindow>;

/// Opens under the toolbar, over a backdrop that closes it on a click elsewhere.
pub fn settings_panel(view: &ConsoleWindow, cx: &mut Cx) -> Option<AnyElement> {
    if !view.settings_open {
        return None;
    }
    let s = view.settings;
    Some(
        div()
            .id("console-settings-backdrop")
            .absolute()
            .inset_0()
            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.settings_open = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("console-settings")
                    .occlude()
                    .on_click(|_, _, _| {})
                    .absolute()
                    .top(px(96.))
                    .right(px(12.))
                    .w(px(280.))
                    .p(px(8.))
                    .rounded(px(R_LG))
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(caption(t("console-view")))
                    .child(switch(
                        "console-set-time",
                        t("console-setting-time"),
                        s.show_time,
                        |s| s.show_time = !s.show_time,
                        cx,
                    ))
                    .child(switch(
                        "console-set-logger",
                        t("console-setting-logger"),
                        s.show_logger,
                        |s| s.show_logger = !s.show_logger,
                        cx,
                    ))
                    .child(switch(
                        "console-set-thread",
                        t("console-setting-thread"),
                        s.show_thread,
                        |s| s.show_thread = !s.show_thread,
                        cx,
                    ))
                    .child(switch(
                        "console-set-wrap",
                        t("console-setting-wrap"),
                        s.wrap,
                        |s| s.wrap = !s.wrap,
                        cx,
                    ))
                    .child(caption(t("console-setting-size")))
                    .child(
                        div().px(px(8.)).pb(px(4.)).flex().gap(px(4.)).children(
                            ConsoleSettings::FONT_SIZES
                                .map(|size| size_option(size, s.font_size == size, cx)),
                        ),
                    ),
            )
            .into_any_element(),
    )
}

fn caption(text: impl Into<SharedString>) -> AnyElement {
    div()
        .px(px(8.))
        .pt(px(8.))
        .pb(px(4.))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(10.))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(TEXT_MUTED))
        .child(text.into())
        .into_any_element()
}

/// The whole row switches, not just the box: a target the size of a word.
fn switch(
    id: &'static str,
    label: impl Into<SharedString>,
    on: bool,
    flip: fn(&mut ConsoleSettings),
    cx: &mut Cx,
) -> AnyElement {
    div()
        .id(id)
        .h(px(36.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(12.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(
            div()
                .size(px(16.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(R_SM))
                .border_1()
                .border_color(rgb(if on { CTA } else { BORDER }))
                .bg(rgb(if on { CTA } else { BG_INPUT }))
                .when(on, |d| d.child(ic("check", 12., ON_CTA))),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(rgb(TEXT_PRIMARY))
                .child(label.into()),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.change_settings(flip, cx);
            cx.notify();
        }))
        .into_any_element()
}

fn size_option(size: u8, active: bool, cx: &mut Cx) -> AnyElement {
    div()
        .id(("console-size", size as u64))
        .h(px(32.))
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(rgb(if active { BG_CARD_HOV } else { BG_INPUT }))
        .border_1()
        .border_color(rgb(if active { CTA } else { BORDER }))
        .font_family("Courier New")
        .text_size(px(f32::from(size)))
        .text_color(rgb(if active { CTA } else { TEXT_SECONDARY }))
        .child(size.to_string())
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.change_settings(|s| s.font_size = size, cx);
            cx.notify();
        }))
        .into_any_element()
}
