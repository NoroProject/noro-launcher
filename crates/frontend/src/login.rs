//! Minimalistic centered login screen for Noro Launcher.

use crate::components::{cta_button, mascot, pixel_title, Mood};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, Context, FontWeight};
use i18n::t;

type Cx<'a> = Context<'a, LauncherUI>;

pub fn render(ui: &mut LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .bg(rgb(BG_WINDOW))
        .p(px(24.))
        .child(
            div()
                .w_full()
                .max_w(px(420.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(24.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(12.))
                        .child(mascot(
                            if ui.logging_in {
                                Mood::Loading
                            } else {
                                Mood::Happy
                            },
                            130.,
                        ))
                        .child(pixel_title("NORO NETWORK", 26., CTA))
                        .child(
                            div()
                                .text_center()
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(13.))
                                .line_height(px(18.))
                                .text_color(rgb(TEXT_MUTED))
                                .child(t("login-tagline")),
                        ),
                )
                .child(auth_content(ui, cx))
                .child(
                    div()
                        .pt(px(8.))
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(11.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                ),
        )
        .into_any_element()
}

fn auth_content(ui: &mut LauncherUI, cx: &mut Cx) -> AnyElement {
    if ui.startup_checking {
        return div()
            .w_full()
            .py(px(20.))
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(13.))
            .text_color(rgb(TEXT_MUTED))
            .child(t("login-checking"))
            .into_any_element();
    }

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(login_buttons(ui.logging_in, cx))
        .when_some(ui.login_error.clone(), |d, e| d.child(error_panel(e, cx)))
        .into_any_element()
}

/// One button, not a list of providers. The auth methods live on the site;
/// mirroring them here would mean a launcher release every time the operator
/// turns another one on.
fn login_buttons(logging_in: bool, cx: &mut Cx) -> AnyElement {
    if logging_in {
        return waiting_box();
    }

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .id("web-login-btn")
                .h(px(48.))
                .w_full()
                .rounded(px(R_MD))
                .bg(rgb(CTA))
                .hover(|s| s.bg(rgb(CTA_HOV)))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(10.))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.start_login();
                    cx.notify();
                }))
                .child(ic("user", 20., ON_CTA))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(14.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(ON_CTA))
                        .child(t("login-sign-in-web")),
                ),
        )
        .child(
            div()
                .text_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .line_height(px(16.))
                .text_color(rgb(TEXT_MUTED))
                .child(t("login-web-hint")),
        )
        .into_any_element()
}

fn waiting_box() -> AnyElement {
    div()
        .h(px(48.))
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER))
        .rounded(px(R_MD))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(14.))
        .text_color(rgb(TEXT_SECONDARY))
        .child(t("login-waiting"))
        .into_any_element()
}

fn error_panel(text: String, cx: &mut Cx) -> AnyElement {
    div()
        .w_full()
        .p(px(14.))
        .rounded(px(R_MD))
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(ERROR))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(10.))
        .child(
            div()
                .text_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .line_height(px(16.))
                .text_color(rgb(ERROR))
                .child(text),
        )
        .child(cta_button(
            "login-retry-btn",
            Some("rotate-ccw"),
            t("retry"),
            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.start_login();
                cx.notify();
            }),
        ))
        .into_any_element()
}
