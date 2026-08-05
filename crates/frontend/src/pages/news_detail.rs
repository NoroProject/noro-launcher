//! Новость целиком: картинка, полный текст, автор и дата.

use super::common::{panel, Cx, CONTENT_W};
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
        .bg(rgb(CONTENT_FALLBACK))
        .flex()
        .flex_col()
        .child(header(cx))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .px(px(16.))
                .py(px(20.))
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(CONTENT_W))
                        .flex()
                        .flex_col()
                        .min_h_0()
                        .gap(px(16.))
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
                ),
        )
        .into_any_element()
}

/// Шапка чтения: стрелка возврата стоит рядом с заголовком, а не отдельной
/// кнопкой над карточкой — там она висела в пустоте.
fn header(cx: &mut Cx) -> AnyElement {
    div()
        .h(px(72.))
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(8.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .id("news-back")
                .size(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(R_SM))
                .cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff10)))
                .child(ic("arrow-left", 16., TEXT_SECONDARY))
                .on_click(cx.listener(|this, _e, _w, cx| {
                    this.page = Page::News;
                    cx.notify();
                })),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("news-title")),
        )
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
            // Картинку держит контейнер с заданной высотой, а сама она внутри
            // растягивается на него. Заданная высота у самого img в колонке не
            // держалась: снимок расползался и наезжал на заголовок.
            div()
                .w_full()
                .h(px(240.))
                .flex_shrink_0()
                .overflow_hidden()
                .rounded(px(R_SM))
                .child(img(image.clone()).size_full().object_fit(ObjectFit::Cover)),
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
    // Тело новости — markdown: заголовки, списки, выделения и ссылки.
    .child(
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .children(super::markdown::render(&item.body)),
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
