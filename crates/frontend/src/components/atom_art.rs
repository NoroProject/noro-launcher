//! Placeholder logo area for future launcher artwork.

use super::mascot::{mascot, Mood};
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement};

pub fn atom_art() -> AnyElement {
    div()
        .size_full()
        .relative()
        .overflow_hidden()
        .flex()
        .items_center()
        .justify_center()
        .p(px(40.))
        .child(hero_showcase())
        .into_any_element()
}

pub fn tiny_atom_logo() -> AnyElement {
    div()
        .px(px(12.))
        .py(px(6.))
        .rounded(px(R_SM))
        .bg(rgba(0xe85aa520))
        .border_1()
        .border_color(rgb(CTA))
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().w(px(10.)).h(px(10.)).rounded_full().bg(rgb(CTA)))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child("NORO LAUNCHER"),
        )
        .into_any_element()
}

fn hero_showcase() -> AnyElement {
    div()
        .w_full()
        .max_w(px(420.))
        .p(px(24.))
        .rounded(px(R_SM))
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgba(0x13233d88))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(16.))
        .child(
            mascot(Mood::Happy, 120.)
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(22.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child("NORO NETWORK"),
        )
        .child(
            div()
                .text_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child("Присоединяйтесь к нашим Minecraft серверам с автоматической установкой модов и синхронизацией скинов."),
        )
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(badge("⚡ 1.21.1"))
                .child(badge("🔑 WebAuthn"))
                .child(badge("🎨 Custom Skins")),
        )
        .into_any_element()
}

fn badge(text: &'static str) -> AnyElement {
    div()
        .px(px(8.))
        .py(px(3.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(BORDER))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(10.))
        .text_color(rgb(TEXT_SECONDARY))
        .child(text)
        .into_any_element()
}
