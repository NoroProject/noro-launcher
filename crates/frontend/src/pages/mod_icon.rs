//! Optional mod icon with local fallback detection.
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::OptionalModInfo;
use gpui::{div, img, prelude::*, px, rgb, rgba, AnyElement, FontWeight, ObjectFit, SharedString};

pub fn category_color(category: &str) -> u32 {
    let l = category.to_lowercase();
    if l.contains("perform") || l.contains("optim") {
        SUCCESS
    } else if l.contains("visual") || l.contains("shader") {
        BLUE
    } else if l.contains("gameplay") || l.contains("game") || l.contains("build") {
        WARNING
    } else {
        ACCENT
    }
}

pub fn mod_icon(ui: &LauncherUI, m: &OptionalModInfo, color: u32) -> AnyElement {
    let base = div()
        .size(px(44.))
        .rounded(px(R_SM))
        .overflow_hidden()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center();

    if let Some(image) = m
        .icon_url
        .as_ref()
        .and_then(|url| ui.optional_mod_icons.get(url))
        .cloned()
    {
        return base
            .child(img(image).size_full().object_fit(ObjectFit::Cover))
            .into_any_element();
    }

    base.bg(rgba((color << 8) | 0x22))
        .border_1()
        .border_color(rgba((color << 8) | 0x66))
        .child(ic(fallback_icon(m), 18., color))
        .into_any_element()
}

pub fn mod_text(m: &OptionalModInfo, width: Option<f32>, desc_chars: usize) -> AnyElement {
    let sub: SharedString = match &m.author {
        Some(a) if !a.is_empty() => format!("{} {a}", i18n::t("mods-by")).into(),
        _ if !m.description.is_empty() => m
            .description
            .chars()
            .take(desc_chars)
            .collect::<String>()
            .into(),
        _ => m.category.clone().into(),
    };

    div()
        .when_some(width, |d, w| d.w(px(w)))
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(14.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .truncate()
                .child(m.name.clone()),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .truncate()
                .child(sub),
        )
        .into_any_element()
}

fn fallback_icon(m: &OptionalModInfo) -> &'static str {
    let haystack = format!("{} {} {}", m.name, m.category, m.description).to_lowercase();
    if has_any(&haystack, &["axiom", "build", "building", "schematic"]) {
        "square"
    } else if has_any(
        &haystack,
        &["perform", "optim", "sodium", "lithium", "ferrite"],
    ) {
        "monitor"
    } else if has_any(
        &haystack,
        &["visual", "shader", "render", "iris", "texture"],
    ) {
        "eye"
    } else if has_any(&haystack, &["voice", "mic"]) {
        "mic"
    } else if has_any(&haystack, &["audio", "sound", "music"]) {
        "volume-2"
    } else if has_any(&haystack, &["lib", "api", "cloth", "kotlin", "config"]) {
        "code"
    } else if has_any(&haystack, &["map", "world", "journey", "xaero"]) {
        "radio"
    } else if has_any(&haystack, &["inventory", "storage", "folder"]) {
        "folder"
    } else {
        "layers"
    }
}

fn has_any(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

/// The name lowercased and without punctuation: this is what a catalog mod
/// and what already sits in the build are compared by.
pub fn normalized(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

/// Whether this mod is part of the build itself.
///
/// Computes only the title key and looks it up in a ready set. This used to
/// walk every file in the build and build a new string for each, for every
/// card on every frame. Twenty cards over a couple of hundred files meant
/// hundreds of thousands of allocations a second and 145 ms a frame.
pub fn is_mod_installed(ui: &LauncherUI, server_id: uuid::Uuid, hit_title: &str) -> bool {
    let key = normalized(hit_title);
    if key.is_empty() {
        return false;
    }
    let Some(keys) = ui.installed_keys.get(&server_id) else {
        return false;
    };
    keys.contains(&key) || keys.iter().any(|k| k.starts_with(&key))
}
