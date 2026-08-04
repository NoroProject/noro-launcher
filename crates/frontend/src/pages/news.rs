use super::common::{page_title, panel, tabs, Cx};
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, FontWeight};
use i18n::t;
use schema::NewsItem;

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
                .bottom(px(32.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(page_title(t("news-title")))
                .children(cards(&ui.news)),
        )
        .into_any_element()
}

fn cards(items: &[NewsItem]) -> Vec<AnyElement> {
    if items.is_empty() {
        return vec![panel()
            .p(px(20.))
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(16.))
            .text_color(rgb(TEXT_MUTED))
            .child(t("news-empty"))
            .into_any_element()];
    }
    items.iter().map(card).collect()
}

fn card(item: &NewsItem) -> AnyElement {
    panel()
        .p(px(20.))
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
                ),
        )
        .child(
            div()
                .line_clamp(4)
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(16.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(item.body.clone()),
        )
        .into_any_element()
}
