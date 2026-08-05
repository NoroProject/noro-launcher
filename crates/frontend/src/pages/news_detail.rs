//! Новость целиком: картинка, полный текст, автор и дата.

use super::common::{panel, tabs, Cx};
use crate::icons::ic;
use crate::state::{LauncherUI, Page};
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, rgba, AnyElement, FontWeight, ObjectFit};
use i18n::t;
use schema::NewsItem;
use uuid::Uuid;

pub fn page(ui: &LauncherUI, id: Uuid, cx: &mut Cx) -> AnyElement {
    let item = ui.news.iter().find(|n| n.id == id);

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
                .bottom(px(32.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(back_button(cx))
                .child(match item {
                    Some(item) => body(ui, item),
                    // Новость могли удалить на мастере, пока её читали.
                    None => panel()
                        .p(px(20.))
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(16.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(t("news-empty"))
                        .into_any_element(),
                }),
        )
        .into_any_element()
}

fn back_button(cx: &mut Cx) -> AnyElement {
    div()
        .id("news-back")
        .flex()
        .items_center()
        .gap(px(8.))
        .w(px(120.))
        .h(px(32.))
        .px(px(12.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(14.))
        .text_color(rgb(TEXT_SECONDARY))
        .hover(|d| d.bg(rgba(0xffffff10)))
        .child(ic("arrow-left", 14., TEXT_SECONDARY))
        .child(t("news-back"))
        .on_click(cx.listener(|this, _e, _w, cx| {
            this.page = Page::News;
            cx.notify();
        }))
        .into_any_element()
}

fn body(ui: &LauncherUI, item: &NewsItem) -> AnyElement {
    let mut root = panel()
        .id("news-body")
        .p(px(24.))
        .flex()
        .flex_col()
        .gap(px(16.))
        .overflow_y_scroll();

    if let Some(image) = ui.news_images.get(&item.id) {
        root = root.child(
            img(image.clone())
                .w_full()
                .h(px(240.))
                .rounded(px(R_SM))
                .object_fit(ObjectFit::Cover),
        );
    }

    root.child(
        div()
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(24.))
            .font_weight(FontWeight::EXTRA_BOLD)
            .text_color(rgb(TEXT_PRIMARY))
            .child(item.title.clone()),
    )
    .child(meta(item))
    .child(
        div()
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(16.))
            .text_color(rgb(TEXT_SECONDARY))
            .child(item.body.clone()),
    )
    .into_any_element()
}

fn meta(item: &NewsItem) -> AnyElement {
    let mut line = item.published_at.format("%Y-%m-%d").to_string();
    if let Some(author) = &item.author_name {
        line.push_str(" · ");
        line.push_str(author);
    }
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(13.))
        .text_color(rgb(TEXT_MUTED))
        .child(line)
        .into_any_element()
}
