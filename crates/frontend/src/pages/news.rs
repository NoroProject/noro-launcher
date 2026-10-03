use super::common::{page_header, panel, Cx, CONTENT_W};
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use i18n::t;
use schema::NewsItem;

pub fn page(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .size_full()
        .bg(rgb(CONTENT_FALLBACK))
        .flex()
        .flex_col()
        .child(page_header("newspaper", t("news-title")))
        .child(
            div()
                // Without a scroll container the posts past the first screen
                // were simply out of reach.
                .id("news-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
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
                        .gap(px(16.))
                        .children(cards(ui, cx)),
                ),
        )
        .into_any_element()
}

fn cards(ui: &LauncherUI, cx: &mut Cx) -> Vec<AnyElement> {
    let items = &ui.news;
    if items.is_empty() {
        return vec![panel()
            .p(px(20.))
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(16.))
            .text_color(rgb(TEXT_MUTED))
            .child(if ui.news_loaded {
                t("news-empty")
            } else {
                t("launcher-loading")
            })
            .into_any_element()];
    }
    items
        .iter()
        .map(|item| {
            card(
                item,
                ui.news_excerpts.get(&item.id).cloned().unwrap_or_default(),
                cx,
            )
        })
        .collect()
}

fn card(item: &NewsItem, excerpt: gpui::SharedString, cx: &mut Cx) -> AnyElement {
    let id = item.id;
    panel()
        // The id has to be unique within the frame, or GPUI merges the click
        // handlers of different cards.
        .id(gpui::ElementId::Name(id.to_string().into()))
        .p(px(20.))
        .cursor_pointer()
        .hover(|d| d.bg(rgba(0xffffff08)))
        .on_click(cx.listener(move |this, _e, _w, cx| {
            this.open_news(id, cx);
            cx.notify();
        }))
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.))
                .child(
                    div()
                        .text_lg()
                        .font_family(FONT_PIXEL_ALT)
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(item.title.clone()),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_xs()
                        .font_family(FONT_PIXEL_ALT)
                        .text_color(rgb(TEXT_MUTED))
                        .child(item.published_at.format("%Y-%m-%d").to_string()),
                )
                .child(
                    div()
                        .ml(px(12.))
                        .text_xs()
                        .font_family(FONT_PIXEL_ALT)
                        .text_color(rgb(CTA))
                        .child(t("news-read")),
                ),
        )
        .child(
            div()
                .line_clamp(3)
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(16.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(excerpt),
        )
        .into_any_element()
}
