//! Mod description body: Modrinth markdown or CurseForge HTML.

use super::mod_detail_parts::meta_row;
use bridge::CatalogHitInfo;
use gpui::{div, prelude::*, px, AnyElement};

pub fn description(hit: &CatalogHitInfo, project: Option<&bridge::ModProjectInfo>) -> AnyElement {
    let body = project.map(|p| p.body.clone()).unwrap_or_default();
    let blocks = if body.trim().is_empty() {
        // Full page hasn't arrived yet; stand in with the search result line.
        vec![super::markdown::render(&hit.description)]
    } else if hit.provider == "curseforge" {
        // CurseForge serves rendered HTML; the markdown parser would let the
        // tags through as text.
        vec![super::markdown::render(&html_to_text(&body))]
    } else {
        vec![super::markdown::render(&body)]
    };

    div()
        .id("mod-description-scroll")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(12.))
        .children(project.map(meta_row))
        .children(blocks.into_iter().flatten())
        .child(div().h(px(16.)))
        .into_any_element()
}

/// Crude HTML to text: tags are dropped, block-level ones become a newline.
pub(super) fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' => {
                in_tag = false;
                let name = tag
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                if matches!(name, "p" | "br" | "div" | "li" | "h1" | "h2" | "h3" | "tr") {
                    out.push('\n');
                }
            }
            _ if in_tag => tag.push(ch),
            _ => out.push(ch),
        }
    }
    out
}
