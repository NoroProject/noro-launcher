//! Server info block: name, description, badges, online count.
use crate::components::badge;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use i18n::t;
use schema::{Modloader, ServerEntry};

pub fn info_block(server: &ServerEntry) -> AnyElement {
    div()
        .absolute()
        .left(px(32.))
        .right(px(240.))
        .bottom(px(132.))
        .flex()
        .flex_col()
        .gap(px(10.))
        .child(name_row(server))
        .when(!server.description.is_empty(), |d| {
            d.child(description_line(server))
        })
        .child(tag_row(server))
        // With one node the breakdown just repeats the status line.
        .when(server.game_servers.len() > 1, |d| d.child(node_row(server)))
        .into_any_element()
}

fn node_row(server: &ServerEntry) -> AnyElement {
    let mut row = div().flex().items_center().flex_wrap().gap(px(8.));
    for node in &server.game_servers {
        let color = if node.live { TEXT_MUTED } else { 0x6b7a99 };
        let value = if node.live {
            format!("{}/{}", node.online, node.max_online)
        } else {
            t("game-node-offline")
        };
        row = row.child(
            div()
                .flex()
                .items_center()
                .gap(px(4.))
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(color))
                .child(div().size(px(6.)).rounded_full().bg(rgb(if node.live {
                    SUCCESS
                } else {
                    0x6b7a99
                })))
                .child(format!("{} {}", node.name.to_uppercase(), value)),
        );
    }
    row.into_any_element()
}

fn name_row(server: &ServerEntry) -> AnyElement {
    let (status_text, dot_color) = status(server);
    div()
        .flex()
        .items_end()
        .gap(px(16.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(30.))
                .font_weight(FontWeight::EXTRA_BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .child(server.name.to_uppercase()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .px(px(8.))
                .py(px(4.))
                .mb(px(4.))
                .rounded(px(R_SM))
                // The banner behind this is an arbitrary image; without a
                // backdrop the online line disappears over its light areas.
                .bg(rgba((OVERLAY << 8) | 0xcc))
                .child(div().size(px(8.)).rounded_full().bg(rgb(dot_color)))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(11.))
                        .text_color(rgb(dot_color))
                        .child(status_text),
                ),
        )
        .into_any_element()
}

fn status(server: &ServerEntry) -> (String, u32) {
    match server.online {
        Some(online) => {
            let max = server
                .max_online
                .map(|m| format!("/{m}"))
                .unwrap_or_default();
            (format!("{online}{max} online"), SUCCESS)
        }
        // No agent reported in. With no game servers configured at all the
        // count is unknown rather than zero, so don't claim offline.
        None if server.game_servers.is_empty() => (t("game-online-unknown"), TEXT_MUTED),
        None => (t("game-node-offline"), ERROR),
    }
}

fn description_line(server: &ServerEntry) -> AnyElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(13.))
        .text_color(rgba(0xc8d6f0a0))
        .max_w(px(520.))
        .child(server.description.chars().take(120).collect::<String>())
        .into_any_element()
}

fn tag_row(server: &ServerEntry) -> AnyElement {
    let (loader_label, loader_color): (&'static str, u32) = match &server.modloader {
        Modloader::Fabric => ("FABRIC", 0xdbb2ff),
        Modloader::Forge => ("FORGE", WARNING),
        Modloader::NeoForge => ("NEOFORGE", 0xff9966),
        Modloader::Quilt => ("QUILT", 0x7ae7c7),
        Modloader::Vanilla => ("VANILLA", SUCCESS),
    };
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(badge(loader_label, loader_color))
        .child(badge(server.mc_version.clone(), BLUE))
        .when(server.limited, |d| {
            d.child(badge(t("game-vip-only"), WARNING))
        })
        .into_any_element()
}
