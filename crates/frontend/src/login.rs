//! ATOM-style login screen.

use crate::components::{
    atom_art, cta_button, mascot, pixel_title, tiny_atom_logo, version_badge, Mood,
};
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, Context, FontWeight};
use i18n::t;

type Cx<'a> = Context<'a, LauncherUI>;

pub fn render(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .size_full()
        .relative()
        .flex()
        .bg(rgb(BG_WINDOW))
        .child(left_panel(ui, cx))
        .child(div().flex_1().min_w_0().child(atom_art()))
        .into_any_element()
}

fn left_panel(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    if ui.startup_checking {
        return div()
            .w(px(560.))
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(20.))
            // Проверка сессии занимает секунды, и всё это время экран был пустым
            // с одной строкой посередине. Маскот показывает, что лаунчер жив.
            .child(mascot(Mood::Loading, 148.))
            .child(pixel_title("NORO", 28., CTA))
            .child(
                div()
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(16.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(t("login-checking")),
            )
            .into_any_element();
    }

    div()
        .w(px(560.))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .px(px(80.))
        .pt(px(40.))
        .pb(px(56.))
        .child(tiny_atom_logo())
        .child(title())
        .child(discord_slot())
        .child(login_action(ui.logging_in, cx))
        .child(login_checks())
        .when_some(ui.login_error.clone(), |d, e| d.child(error_panel(e, cx)))
        .child(div().flex_1())
        .child(version_badge())
        .into_any_element()
}

fn title() -> AnyElement {
    div()
        .mt(px(96.))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(36.))
        .line_height(px(44.))
        .text_color(rgb(CTA))
        .child(t("login-title"))
        .into_any_element()
}

fn discord_slot() -> AnyElement {
    div()
        .mt(px(52.))
        .mb(px(32.))
        .h(px(64.))
        .w_full()
        .flex()
        .items_center()
        .px(px(28.))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(0x0a1424))
        .text_color(rgb(TEXT_MUTED))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(20.))
        .child(t("login-subtitle"))
        .into_any_element()
}

fn login_action(logging_in: bool, cx: &mut Cx) -> AnyElement {
    if logging_in {
        return div()
            .h(px(56.))
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(BG_CARD))
            .border_2()
            .border_color(rgb(BORDER))
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(18.))
            .text_color(rgb(TEXT_SECONDARY))
            .child(t("login-waiting"))
            .into_any_element();
    }

    cta_button(
        "discord-login",
        Some("user"),
        t("login-sign-in"),
        cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.start_login();
            cx.notify();
        }),
    )
    .into_any_element()
}

fn login_checks() -> AnyElement {
    div()
        .mt(px(40.))
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(check_line(t("login-save-session"), true))
        .child(check_line(t("login-auto-login"), false))
        .into_any_element()
}

fn check_line(label: impl Into<gpui::SharedString>, checked: bool) -> AnyElement {
    let label = label.into();
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .text_color(rgb(TEXT_SECONDARY))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(16.))
        .font_weight(FontWeight::BOLD)
        .child(
            div()
                .size(px(20.))
                .border_2()
                .border_color(rgb(CTA))
                .bg(rgb(if checked { CTA } else { BG_WINDOW })),
        )
        .child(label)
        .into_any_element()
}

fn error_panel(text: String, cx: &mut Cx) -> AnyElement {
    div()
        .mt(px(24.))
        .p(px(16.))
        .rounded(px(R_SM))
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(ERROR))
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(14.))
                .text_color(rgb(ERROR))
                .child(text),
        )
        .child(
            cta_button(
                "login-retry-btn",
                Some("rotate-ccw"),
                t("retry"),
                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.start_login();
                    cx.notify();
                }),
            ),
        )
        .into_any_element()
}
