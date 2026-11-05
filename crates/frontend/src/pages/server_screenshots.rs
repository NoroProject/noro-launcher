// Over 150 lines: the page, its header and the card are one layout and read
// together.
//! SCREENSHOTS tab: the game's `screenshots` folder as a gallery.
use super::common::{panel, short_date, tabs, Cx};
use crate::icons::ic;
use crate::state::{LauncherUI, GALLERY_PAGE};
use crate::theme::*;
use bridge::ScreenshotInfo;
use gpui::{
    div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, ObjectFit,
    SharedString,
};
use i18n::t;
use uuid::Uuid;

const CARD_W: f32 = 240.;
const THUMB_H: f32 = 136.;

pub fn page(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let visible: Vec<_> = ui
        .gallery
        .items
        .iter()
        .take(ui.gallery.shown.max(GALLERY_PAGE))
        .map(|s| s.path.clone())
        .collect();
    for path in &visible {
        ui.ensure_screenshot_thumb(path, cx);
    }
    let ui = &*ui;

    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(104.))
                .left(px(32.))
                .right(px(32.))
                .bottom(px(32.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(header(ui, server_id, cx))
                .child(body(ui, server_id, cx)),
        )
        .into_any_element()
}

fn header(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let mut args = i18n::FluentArgs::new();
    args.set("count", ui.gallery.items.len());
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(ic("image", 20., ACCENT))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("screenshots-title")),
        )
        .when(ui.gallery.loaded, |d| {
            d.child(
                div()
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(12.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(i18n::t_args("screenshots-count", &args)),
            )
        })
        .child(div().flex_1())
        .child(action(
            "shots-refresh",
            "refresh",
            t("screenshots-refresh"),
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.refresh_screenshots(server_id);
                cx.notify();
            }),
        ))
        .child(action(
            "shots-folder",
            "folder",
            t("screenshots-open-folder"),
            cx.listener(move |this, _e: &ClickEvent, _w, _cx| {
                this.open_screenshots_folder(server_id);
            }),
        ))
        .into_any_element()
}

fn action(
    id: &'static str,
    icon: &'static str,
    label: String,
    on_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .h(px(32.))
        .px(px(12.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .bg(rgba(0xffffff0a))
        .border_1()
        .border_color(rgb(BORDER))
        .hover(|s| s.bg(rgba(0xffffff14)).border_color(rgb(ACCENT)))
        .child(ic(icon, 12., ACCENT))
        .child(
            div()
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(label),
        )
        .on_click(on_click)
        .into_any_element()
}

fn body(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let g = &ui.gallery;
    if !g.loaded || g.items.is_empty() {
        return panel()
            .p(px(32.))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.))
            .child(ic("image", 28., TEXT_MUTED))
            .child(
                div()
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(14.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(if g.loaded {
                        t("screenshots-empty")
                    } else {
                        t("launcher-loading")
                    }),
            )
            .into_any_element();
    }

    let shown = g.shown.max(GALLERY_PAGE).min(g.items.len());
    let cards: Vec<AnyElement> = g.items[..shown]
        .iter()
        .map(|s| card(ui, server_id, s, cx))
        .collect();

    div()
        .id("shots-scroll")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(div().flex().flex_wrap().gap(px(16.)).children(cards))
        .when(shown < g.items.len(), |d| {
            d.child(div().flex().justify_center().pb(px(8.)).child(action(
                "shots-more",
                "chevron-down",
                t("screenshots-show-more"),
                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.show_more_screenshots();
                    cx.notify();
                }),
            )))
        })
        .into_any_element()
}

fn card(ui: &LauncherUI, server_id: Uuid, shot: &ScreenshotInfo, cx: &mut Cx) -> AnyElement {
    let thumb = ui.gallery.thumbs.get(&shot.path).cloned();
    let armed_key = format!("shot-del-{}", shot.file_name);
    let armed = ui.is_armed(&armed_key);
    let (open_name, del_name) = (shot.file_name.clone(), shot.file_name.clone());

    let picture = div()
        .id(SharedString::from(format!("shot-open-{}", shot.file_name)))
        .w(px(CARD_W))
        .h(px(THUMB_H))
        .bg(rgba(0x00000040))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .child(match thumb {
            Some(image) => img(image)
                .size_full()
                .object_fit(ObjectFit::Cover)
                .into_any_element(),
            None => ic("image", 20., TEXT_MUTED).into_any_element(),
        })
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, _cx| {
            this.open_screenshot(server_id, open_name.clone());
        }));

    let delete = div()
        .id(SharedString::from(armed_key.clone()))
        .h(px(28.))
        .px(px(8.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(4.))
        .cursor_pointer()
        .hover(|s| s.bg(rgba(0xffffff14)))
        .child(ic("trash-2", 12., if armed { ERROR } else { TEXT_MUTED }))
        .when(armed, |d| {
            d.child(
                div()
                    .text_size(px(10.))
                    .text_color(rgb(ERROR))
                    .child(t("common-click-again")),
            )
        })
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            if this.confirm_or_arm(armed_key.clone(), cx) {
                this.delete_screenshot(server_id, del_name.clone());
            }
            cx.notify();
        }));

    panel()
        .w(px(CARD_W))
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(picture)
        .child(
            div()
                .h(px(36.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(11.))
                        .text_color(rgb(TEXT_SECONDARY))
                        .child(short_date(shot.taken_at)),
                )
                .child(delete),
        )
        .into_any_element()
}
