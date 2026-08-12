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
        .px(px(64.))
        .pt(px(40.))
        .pb(px(40.))
        .child(tiny_atom_logo())
        .child(title())
        .child(subtitle())
        .child(login_action(ui.logging_in, cx))
        .when_some(ui.login_error.clone(), |d, e| d.child(error_panel(e, cx)))
        .child(div().flex_1())
        .child(version_badge())
        .into_any_element()
}

fn title() -> AnyElement {
    div()
        .mt(px(48.))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(32.))
        .line_height(px(40.))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(CTA))
        .child("АВТОРИЗАЦИЯ")
        .into_any_element()
}

fn subtitle() -> AnyElement {
    div()
        .mt(px(8.))
        .mb(px(32.))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(13.))
        .text_color(rgb(TEXT_MUTED))
        .child("Выберите способ входа в профиль игрока Noro Network:")
        .into_any_element()
}

fn login_action(logging_in: bool, cx: &mut Cx) -> AnyElement {
    if logging_in {
        return div()
            .h(px(52.))
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(BG_CARD))
            .border_2()
            .border_color(rgb(BORDER))
            .rounded(px(R_SM))
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(15.))
            .text_color(rgb(TEXT_SECONDARY))
            .child(t("login-waiting"))
            .into_any_element();
    }

    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .w_full()
        .child(cta_button(
            "discord-login",
            Some("user"),
            "Войти через Discord",
            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.start_login();
                cx.notify();
            }),
        ))
        .child(cta_button(
            "passkey-login",
            Some("key-round"),
            "Войти через Passkey",
            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.start_login();
                cx.notify();
            }),
        ))
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
