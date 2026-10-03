//! Profile page: skin model preview on the left, sub-tabbed column on the right.

use super::common::{page_title, panel, tabs, Cx};
use super::profile_cape::cape_panel;
use super::profile_skin::skin_card;
use super::profile_skin_presets::skin_presets_panel;
use super::profile_user::user_card;
use super::skin_drag;
use crate::state::{LauncherUI, ProfileTab};
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent};
use i18n::t;

pub fn page(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(104.))
                .left(px(40.))
                .right(px(40.))
                .bottom(px(40.))
                .flex()
                .flex_col()
                .gap(px(20.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(page_title(t("profile-title")))
                        .when(ui.user.is_some(), |d| d.child(sub_tabs_bar(ui, cx))),
                )
                .child(match ui.user.clone() {
                    Some(_) => content(ui, cx),
                    None => empty_state(),
                }),
        )
        .when(ui.skin_dragging, |d| d.child(skin_drag::drag_overlay(cx)))
        .when(ui.cape_selector_open, |d| {
            d.child(super::profile_cape::cape_modal(ui, cx))
        })
        .into_any_element()
}

fn empty_state() -> AnyElement {
    panel()
        .p(px(20.))
        .text_color(rgb(TEXT_MUTED))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(14.))
        .child(t("profile-unavailable"))
        .into_any_element()
}

fn content(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .flex_1()
        .min_h_0()
        .h_full()
        .flex()
        .items_stretch()
        .gap(px(24.))
        .child(skin_card(ui, cx))
        .child(right_column(ui, cx))
        .into_any_element()
}

fn right_column(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let user = ui.user.as_ref().expect("user in profile");

    div()
        .w(gpui::relative(0.6))
        .min_w_0()
        .min_h_0()
        .h_full()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(match ui.profile_tab {
            ProfileTab::Skins => skin_presets_panel(ui, cx),
            ProfileTab::Capes => cape_panel(ui, cx),
            _ => user_card(ui, user, cx),
        })
        .into_any_element()
}

fn sub_tabs_bar(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let current = ui.profile_tab;
    let (overview, capes, presets) = (
        t("profile-tab-overview"),
        t("profile-tab-capes"),
        t("profile-tab-presets"),
    );
    div()
        .flex()
        .gap(px(8.))
        .child(sub_tab_button(
            "tab-main",
            overview,
            current == ProfileTab::Overview,
            |this, cx| {
                this.profile_tab = ProfileTab::Overview;
                cx.notify();
            },
            cx,
        ))
        .child(sub_tab_button(
            "tab-capes",
            capes,
            current == ProfileTab::Capes,
            |this, cx| {
                this.profile_tab = ProfileTab::Capes;
                cx.notify();
            },
            cx,
        ))
        .child(sub_tab_button(
            "tab-presets",
            presets,
            current == ProfileTab::Skins,
            |this, cx| {
                this.profile_tab = ProfileTab::Skins;
                this.load_preset_renders(cx);
                this.backend
                    .send(bridge::MessageToBackend::RequestSkinPresetsList);
                cx.notify();
            },
            cx,
        ))
        .into_any_element()
}

fn sub_tab_button(
    id: &'static str,
    label: impl Into<gpui::SharedString>,
    active: bool,
    on_click: impl Fn(&mut LauncherUI, &mut Cx) + 'static,
    cx: &mut Cx,
) -> AnyElement {
    crate::components::segment(
        id,
        None,
        label,
        active,
        cx.listener(move |this, _e: &ClickEvent, _w, cx| on_click(this, cx)),
    )
}
