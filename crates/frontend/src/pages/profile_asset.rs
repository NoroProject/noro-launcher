use super::common::Cx;
use crate::components::btn;
use crate::icons::ic;
use crate::state::Toast;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, FontWeight, SharedString};
use i18n::t;
use schema::NotifLevel;

/// `id` is separate from `title` because the title is translated and the
/// element id has to stay the same in every language.
pub fn asset_panel(
    id: &'static str,
    title: impl Into<SharedString>,
    value: Option<&str>,
    profile_url: &str,
    cx: &mut Cx,
) -> AnyElement {
    let open_url = profile_url.to_string();
    panel_row()
        .child(icon_box())
        .child(text_block(title.into(), value))
        .child(btn(
            SharedString::from(id),
            t("profile-edit"),
            true,
            cx.listener(move |this, _e, _w, cx| {
                if let Err(err) = open::that(open_url.clone()) {
                    this.toast = Some(Toast {
                        text: format!("Cannot open cabinet: {err}"),
                        level: NotifLevel::Error,
                    });
                }
                cx.notify();
            }),
        ))
        .into_any_element()
}

fn panel_row() -> gpui::Div {
    div()
        .p(px(16.))
        .flex()
        .items_center()
        .gap(px(16.))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .rounded(px(R_SM))
}

fn icon_box() -> AnyElement {
    div()
        .size(px(56.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .flex()
        .items_center()
        .justify_center()
        .child(ic("image", 24., TEXT_SECONDARY))
        .into_any_element()
}

fn text_block(title: SharedString, value: Option<&str>) -> AnyElement {
    div()
        .min_w_0()
        .flex_1()
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(16.))
                .font_weight(FontWeight::BOLD)
                .child(title.to_uppercase()),
        )
        .child(
            div()
                .truncate()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(
                    value
                        .map(str::to_uppercase)
                        .unwrap_or_else(|| t("profile-not-set")),
                ),
        )
        .into_any_element()
}
