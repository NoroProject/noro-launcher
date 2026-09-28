//! Parts of the mod page: tabs, screenshot gallery, metadata row.

use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, ObjectFit, Window};

pub fn tab_button(
    id: &'static str,
    label: &str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    crate::components::segment(id, None, label.to_string(), active, on_click)
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

/// Facts about the project, grouped by what they are.
///
/// They used to be one run of identical chips — categories, loaders, three
/// game versions and a licence identifier side by side — and a licence called
/// `LicenseRef-Polyform-Shield-1.0.0` next to `fabric` reads as noise. Now the
/// row carries what people scan for, and the rest goes underneath as text.
pub fn meta_row(project: &bridge::ModProjectInfo) -> AnyElement {
    let mut chips: Vec<AnyElement> = Vec::new();
    for c in project.categories.iter().take(4) {
        chips.push(chip(c, ACCENT));
    }
    for l in project.loaders.iter().take(3) {
        chips.push(chip(l, BLUE));
    }

    // A project can list a hundred versions. The range is the useful part: the
    // oldest and the newest say what it runs on, a sample of three does not.
    let versions = match (project.game_versions.first(), project.game_versions.last()) {
        (Some(first), Some(last)) if first != last => Some(format!("{first} – {last}")),
        (Some(only), _) => Some(only.clone()),
        _ => None,
    };
    let license = project.license.clone().filter(|s| !s.is_empty());

    if chips.is_empty() && versions.is_none() && license.is_none() {
        return div().into_any_element();
    }

    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .pb(px(4.))
        .when(!chips.is_empty(), |d| {
            d.child(div().flex().flex_wrap().gap(px(6.)).children(chips))
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(14.))
                .text_size(px(11.))
                .text_color(rgb(TEXT_MUTED))
                .children(versions.map(|v| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .child(ic("box", 11., TEXT_MUTED))
                        .child(v)
                }))
                .children(license.map(|l| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .min_w_0()
                        .child(ic("lock", 11., TEXT_MUTED))
                        .child(div().truncate().child(l))
                })),
        )
        .into_any_element()
}

fn chip(text: &str, color: u32) -> AnyElement {
    div()
        .px(px(8.))
        .py(px(3.))
        .rounded(px(R_SM))
        .bg(rgba((color << 8) | 0x20))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(11.))
        .text_color(rgb(color))
        .child(text.to_string())
        .into_any_element()
}
