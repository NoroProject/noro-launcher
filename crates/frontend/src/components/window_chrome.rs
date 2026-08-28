use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, Context, FontWeight, MouseButton, MouseDownEvent,
    Pixels, Point, Size, WindowControlArea,
};
use i18n::{t, Locale};

/// `compact` is the thinner bar used once the launcher is past login.
pub fn window_chrome(compact: bool, ui: &LauncherUI, cx: &mut Context<LauncherUI>) -> AnyElement {
    let active = ui.locale;
    div()
        .id("window-chrome")
        .h(px(if compact { 36. } else { 48. }))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .px(px(16.))
        .gap(px(4.))
        // The bar runs into the top edge of the window, where the system resize
        // zone lives. Starting a move in there fights the resize: the cursor
        // flickers, nothing resizes, and AppKit complains about a move that
        // completed without beginning.
        .on_mouse_down(MouseButton::Left, |event: &MouseDownEvent, window, _| {
            if !in_resize_edge(event.position, window.viewport_size()) {
                window.start_window_move();
            }
        })
        .children(impersonate_pill(ui, cx))
        // Windows drags the window itself, from its answer to WM_NCHITTEST;
        // `start_window_move()` does nothing there. Only the empty stretch is
        // marked — cover the whole bar and the system treats the buttons as
        // title bar and eats clicks on them.
        .child(
            div()
                .flex_1()
                .h_full()
                .window_control_area(WindowControlArea::Drag),
        )
        .children(Locale::ALL.map(|l| lang_pill(l, l == active, cx)))
        .child(div().w(px(8.)))
        .child(control("win-min", "minus", false))
        .child(control("win-close", "x", true))
        .into_any_element()
}

/// "You are in someone else's account", in the chrome rather than as a banner
/// under it — it belongs to the window, and here it doesn't push content down.
fn impersonate_pill(ui: &LauncherUI, cx: &mut Context<LauncherUI>) -> Option<AnyElement> {
    let name = ui.impersonating_as.clone()?;

    Some(
        div()
            .h(px(24.))
            .flex()
            .items_center()
            .gap(px(8.))
            .pl(px(8.))
            .pr(px(4.))
            .rounded(px(R_SM))
            .bg(rgb(ACCENT))
            .child(ic("eye-off", 12., ON_CTA))
            .child(
                div()
                    .text_size(px(12.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(ON_CTA))
                    .child(format!("{} {name}", t("impersonate-banner"))),
            )
            .child(
                div()
                    .id("impersonate-exit")
                    .h(px(20.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .rounded(px(R_SM))
                    .cursor_pointer()
                    .bg(rgba((ON_CTA << 8) | 0x22))
                    .hover(|s| s.bg(rgba((ON_CTA << 8) | 0x44)))
                    .child(ic("log-out", 10., ON_CTA))
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(ON_CTA))
                            .child(t("impersonate-exit")),
                    )
                    .on_click(cx.listener(|this, _e, _w, cx| {
                        this.exit_impersonation();
                        cx.notify();
                    })),
            )
            .into_any_element(),
    )
}

/// macOS takes a resize within a couple of pixels of the edge. This is wider
/// than that on purpose — otherwise hitting the zone with a mouse is a chore.
const RESIZE_EDGE: f32 = 6.;

fn in_resize_edge(position: Point<Pixels>, viewport: Size<Pixels>) -> bool {
    let edge = px(RESIZE_EDGE);
    position.y <= edge || position.x <= edge || position.x >= viewport.width - edge
}

fn lang_pill(locale: Locale, active: bool, cx: &mut Context<LauncherUI>) -> AnyElement {
    div()
        .id(locale.code())
        .px(px(8.))
        .h(px(24.))
        .flex()
        .items_center()
        .rounded(px(R_SM))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(14.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(if active { CTA } else { TEXT_MUTED }))
        .bg(if active {
            rgba((CTA << 8) | 0x18)
        } else {
            rgba(0x00000000)
        })
        .cursor_pointer()
        .hover(|d| d.bg(rgba(0xffffff10)))
        .child(locale.label())
        .on_click(cx.listener(move |this, _e, _w, cx| {
            this.set_locale(locale);
            cx.notify();
        }))
        .into_any_element()
}

fn control(id: &'static str, icon: &'static str, is_close: bool) -> AnyElement {
    div()
        .id(id)
        .size(px(26.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(R_SM))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(if is_close { ERROR } else { BG_CARD_HOV })))
        .child(ic(icon, 14., TEXT_SECONDARY))
        .on_click(move |_, window, cx| {
            if is_close {
                cx.quit();
            } else {
                window.minimize_window();
            }
        })
        .into_any_element()
}
