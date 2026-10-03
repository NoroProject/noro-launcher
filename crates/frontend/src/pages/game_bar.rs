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

/// Where the bar sits. The build list opens above it and lines up with it.
pub const BAR_INSET: f32 = 32.;
pub const BAR_HEIGHT: f32 = 80.;
pub const BAR_PADDING: f32 = 24.;

pub fn bottom_bar(
    ui: &LauncherUI,
    server: &ServerEntry,
    sync: &SyncUiState,
    locked: bool,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .absolute()
        .left(px(BAR_INSET))
        .right(px(BAR_INSET))
        .bottom(px(BAR_INSET))
        .h(px(BAR_HEIGHT))
        .px(px(BAR_PADDING))
        .rounded(px(R_SM))
        .bg(rgba(0x0b1626ec))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .items_center()
        .child(super::build_picker::build_block(ui, server, cx))
        .child(div().flex_1())
        .child(
            div()
                // Flex shrinks children by default, and when the bar got tight
                // the console button gave up width but kept its height.
                .flex_none()
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
        .tooltip(crate::components::hint(i18n::t("hint-console")))
        // Matches the play button beside it.
        .size(px(56.))
        .flex_none()
        .rounded(px(R_SM))
        .cursor_pointer()
        // An active icon button lights up cream, as in the sidebar: magenta here was
        // a third way of saying "on" in a single interface.
        .bg(if active {
            rgba((CTA << 8) | 0x18)
        } else {
            rgb(BG_INPUT)
        })
        .border_1()
        .border_color(rgb(if active { CTA } else { BORDER }))
        .flex()
        .items_center()
        .justify_center()
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(ic(
            "terminal",
            20.,
            if active { CTA } else { TEXT_SECONDARY },
        ))
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.toggle_console(cx);
            cx.notify();
        }))
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
    if sync.cancellable() {
        return cta_button(
            "cancel-launch",
            Some("x"),
            t("common-cancel"),
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.cancel_launch(server_id);
                cx.notify();
            }),
        )
        .into_any_element();
    }
    if sync.syncing {
        return disabled(sync.heading_text());
    }
    if sync.running {
        return stop_button(server_id, sync.stop_armed, cx);
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

/// Stopping the game loses whatever wasn't saved, so it takes two clicks: the
/// first arms the button for a few seconds, the second stops the game.
fn stop_button(server_id: Uuid, armed: bool, cx: &mut Cx) -> AnyElement {
    div()
        .id("stop-game")
        .w(px(216.))
        .h(px(56.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(rgb(if armed { ERROR } else { BG_CARD }))
        .border_1()
        .border_color(rgb(if armed { ERROR } else { BORDER }))
        .hover(move |d| d.bg(rgb(if armed { ERROR } else { BG_CARD_HOV })))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(8.))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(if armed { 14. } else { 18. }))
        .font_weight(FontWeight::EXTRA_BOLD)
        .child(ic("square", 16., TEXT_PRIMARY))
        .child(if armed {
            t("game-stop-confirm")
        } else {
            t("game-stop")
        })
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.stop_clicked(server_id, cx);
            cx.notify();
        }))
        .into_any_element()
}
