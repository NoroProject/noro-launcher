//! Parts of the mod page: tabs, screenshot gallery, metadata row.

use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, ObjectFit,
    SharedString, Window,
};

pub fn tab_button(
    id: &'static str,
    label: &str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(SharedString::from(id))
        .px(px(16.))
        .py(px(8.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(13.))
        .when(active, |d| {
            d.bg(rgb(CTA))
                .text_color(rgb(BG_INPUT))
                .font_weight(FontWeight::BOLD)
        })
        .when(!active, |d| {
            d.text_color(rgb(TEXT_MUTED))
                .hover(|s| s.bg(rgba(0xffffff10)))
        })
        .child(label.to_string())
        .on_click(on_click)
        .into_any_element()
}

/// Screenshots in two columns. They go through the same loader as the icons and
/// land in `optional_mod_icons`, so this only picks up what is already there.
pub fn gallery(ui: &LauncherUI, shots: &[String]) -> AnyElement {
    let mut rows: Vec<AnyElement> = Vec::new();
    for pair in shots.chunks(2) {
        let mut row = div().flex().gap(px(12.));
        for url in pair {
            row = row.child(shot(ui, url));
        }
        rows.push(row.into_any_element());
    }

    div()
        .id("mod-gallery-scroll")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(12.))
        .children(rows)
        .into_any_element()
}

fn shot(ui: &LauncherUI, url: &str) -> AnyElement {
    let frame = div()
        .flex_1()
        .h(px(200.))
        .rounded(px(R_SM))
        .overflow_hidden()
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgb(BG_INPUT))
        .flex()
        .items_center()
        .justify_center();

    match ui.optional_mod_icons.get(url).cloned() {
        Some(image) => frame
            .child(img(image).size_full().object_fit(ObjectFit::Contain))
            .into_any_element(),
        // Keep the frame while it loads, or the grid jumps when it arrives.
        None => frame.child(ic("image", 24., TEXT_MUTED)).into_any_element(),
    }
}

pub fn meta_row(project: &bridge::ModProjectInfo) -> AnyElement {
    let mut chips: Vec<AnyElement> = Vec::new();
    for c in project.categories.iter().take(4) {
        chips.push(chip(c, ACCENT));
    }
    for l in project.loaders.iter().take(3) {
        chips.push(chip(l, BLUE));
    }
    // A project can list close to a hundred versions; only the newest few fit.
    for v in project.game_versions.iter().rev().take(3) {
        chips.push(chip(v, TEXT_MUTED));
    }
    if let Some(license) = project.license.as_ref().filter(|s| !s.is_empty()) {
        chips.push(chip(license, SUCCESS));
    }

    if chips.is_empty() {
        return div().into_any_element();
    }
    div()
        .flex()
        .flex_wrap()
        .gap(px(8.))
        .pb(px(4.))
        .children(chips)
        .into_any_element()
}

fn chip(text: &str, color: u32) -> AnyElement {
    div()
        .px(px(8.))
        .py(px(4.))
        .rounded(px(R_SM))
        .bg(rgba((color << 8) | 0x20))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(11.))
        .text_color(rgb(color))
        .child(text.to_string())
        .into_any_element()
}
