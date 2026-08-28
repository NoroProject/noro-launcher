use super::common::Cx;
use crate::components::cta_button;
use crate::icons::ic;
use crate::state::{LauncherUI, SyncUiState};
use crate::theme::*;
use bridge::BuildState;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use schema::ServerEntry;
use uuid::Uuid;

pub fn bottom_bar(
    ui: &LauncherUI,
    server: &ServerEntry,
    sync: &SyncUiState,
    locked: bool,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .absolute()
        .left(px(32.))
        .right(px(32.))
        .bottom(px(32.))
        .h(px(80.))
        .px(px(24.))
        .rounded(px(R_SM))
        .bg(rgba(0x0b1626ec))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .items_center()
        .child(version_block(server))
        .children(
            super::build_picker::build_picker(ui, server, cx).map(|p| div().ml(px(20.)).child(p)),
        )
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(16.))
                .child(console_button(ui.console_window.is_some(), cx))
                .child(play_button(
                    server.id,
                    sync,
                    locked,
                    ui.build_state.get(&server.id).copied().unwrap_or_default(),
                    cx,
                )),
        )
        .into_any_element()
}

fn console_button(active: bool, cx: &mut Cx) -> AnyElement {
    div()
        .id("toggle-console")
        // Matches the play button beside it.
        .size(px(56.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(if active {
            rgb(BG_CARD_HOV)
        } else {
            rgb(BG_INPUT)
        })
        .border_1()
        .border_color(rgb(if active { ACCENT } else { BORDER }))
        .flex()
        .items_center()
        .justify_center()
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(ic(
            "layers",
            20.,
            if active { ACCENT } else { TEXT_SECONDARY },
        ))
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.toggle_console(cx);
            cx.notify();
        }))
        .into_any_element()
}

fn version_block(server: &ServerEntry) -> AnyElement {
    // A dash rather than "draft": there is no build at all here, and "draft"
    // reads as "there is one, it's just rough".
    let version = server.current_version.as_deref().unwrap_or("—");
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(
            div()
                .size(px(48.))
                .rounded(px(R_SM))
                .bg(rgb(BG_INPUT))
                .border_1()
                .border_color(rgb(BORDER))
                .flex()
                .items_center()
                .justify_center()
                .child(ic("server", 22., CTA)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(9.))
                        .text_color(rgb(TEXT_MUTED))
                        .font_weight(FontWeight::BOLD)
                        .child(t("game-build")),
                )
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(20.))
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(version.to_string()),
                ),
        )
        .into_any_element()
}

fn play_button(
    server_id: Uuid,
    sync: &SyncUiState,
    locked: bool,
    build: BuildState,
    cx: &mut Cx,
) -> AnyElement {
    if locked {
        return disabled(t("game-locked"));
    }
    if sync.syncing {
        return disabled(t("game-preparing"));
    }
    if sync.running {
        return stop_button(server_id, cx);
    }
    if sync.failed.is_some() {
        return cta_button(
            "retry-game",
            Some("rotate-ccw"),
            t("retry"),
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.launch(server_id);
                cx.notify();
            }),
        )
        .into_any_element();
    }
    // Same action every time — sync, then launch. Only the label moves, because
    // calling it "play" with nothing on disk would be a lie.
    let (icon, label) = match build {
        BuildState::Missing => ("download", t("game-install")),
        BuildState::Outdated => ("refresh", t("game-update")),
        BuildState::Ready => ("play", t("game-start")),
    };
    cta_button(
        "start-game",
        Some(icon),
        label,
        cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.launch(server_id);
            cx.notify();
        }),
    )
    .into_any_element()
}

fn disabled(label: impl Into<gpui::SharedString>) -> AnyElement {
    let label = label.into();
    div()
        .w(px(216.))
        .h(px(56.))
        .rounded(px(R_SM))
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(18.))
        .font_weight(FontWeight::EXTRA_BOLD)
        .text_color(rgb(TEXT_MUTED))
        .child(label)
        .into_any_element()
}

fn stop_button(server_id: Uuid, cx: &mut Cx) -> AnyElement {
    div()
        .id("stop-game")
        .w(px(216.))
        .h(px(56.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(8.))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(18.))
        .font_weight(FontWeight::EXTRA_BOLD)
        .child(ic("square", 16., TEXT_PRIMARY))
        .child(t("game-stop"))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.kill(server_id);
            cx.notify();
        }))
        .into_any_element()
}
