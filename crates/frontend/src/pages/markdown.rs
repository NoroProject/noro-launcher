//! Markdown news bodies into GPUI elements.
//!
//! Inline styles go on as ranges over a single `StyledText` rather than as
//! separate elements. Only that way does the paragraph wrap by words; split
//! into elements it breaks at every bold run.

use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, AnyElement, FontStyle, FontWeight, HighlightStyle, StyledText,
};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::ops::Range;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Paragraph,
    Heading(u8),
    Code,
    Quote,
    Item(usize),
}

/// One element per paragraph, heading and list item.
pub fn render(source: &str) -> Vec<AnyElement> {
    let parser = Parser::new_ext(source, Options::ENABLE_STRIKETHROUGH);
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut spans: Vec<(Range<usize>, HighlightStyle)> = Vec::new();
    let mut open: Vec<(usize, HighlightStyle)> = Vec::new();
    let mut kind = Kind::Paragraph;
    let mut depth = 0usize;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => kind = Kind::Heading(heading_size(level)),
            Event::Start(Tag::CodeBlock(_)) => kind = Kind::Code,
            Event::Start(Tag::BlockQuote(_)) => kind = Kind::Quote,
            Event::Start(Tag::List(_)) => depth += 1,
            Event::End(TagEnd::List(_)) => depth = depth.saturating_sub(1),
            Event::Start(Tag::Item) => {
                kind = Kind::Item(depth.saturating_sub(1));
                buf.push_str("• ");
            }
            Event::Start(Tag::Strong) => {
                open.push((buf.len(), style(FontWeight::BOLD, None, None)))
            }
            Event::Start(Tag::Emphasis) => open.push((
                buf.len(),
                style(FontWeight::NORMAL, Some(FontStyle::Italic), None),
            )),
            Event::Start(Tag::Link { .. }) => {
                open.push((buf.len(), style(FontWeight::NORMAL, None, Some(BLUE))))
            }
            Event::End(TagEnd::Strong | TagEnd::Emphasis | TagEnd::Link) => {
                if let Some((start, hl)) = open.pop() {
                    spans.push((start..buf.len(), hl));
                }
            }
            Event::Text(text) => buf.push_str(&text),
            Event::Code(text) => {
                let start = buf.len();
                buf.push_str(&text);
                spans.push((
                    start..buf.len(),
                    style(FontWeight::NORMAL, None, Some(ACCENT)),
                ));
            }
            Event::SoftBreak => buf.push(' '),
            Event::HardBreak => buf.push('\n'),
            Event::Rule => out.push(rule()),
            Event::End(
                TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::CodeBlock | TagEnd::Item,
            ) => {
                flush(&mut out, &mut buf, &mut spans, kind);
                kind = Kind::Paragraph;
            }
            _ => {}
        }
    }
    flush(&mut out, &mut buf, &mut spans, kind);
    out
}

pub fn plain_excerpt(source: &str, limit: usize) -> String {
    let mut out = String::new();
    for event in Parser::new(source) {
        match event {
            Event::Text(text) | Event::Code(text) => out.push_str(&text),
            Event::SoftBreak | Event::HardBreak | Event::End(TagEnd::Paragraph) => out.push(' '),
            _ => {}
        }
        if out.chars().count() > limit {
            break;
        }
    }
    let trimmed: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.chars().count() > limit {
        let cut: String = trimmed.chars().take(limit).collect();
        format!("{}…", cut.trim_end())
    } else {
        trimmed
    }
}

fn heading_size(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 22,
        HeadingLevel::H2 => 19,
        _ => 17,
    }
}

fn style(weight: FontWeight, italic: Option<FontStyle>, color: Option<u32>) -> HighlightStyle {
    HighlightStyle {
        font_weight: Some(weight),
        font_style: italic,
        color: color.map(|c| rgb(c).into()),
        ..Default::default()
    }
}

/// Closes out the accumulated block. Empty ones are dropped, otherwise double
/// line breaks would leave blank strips in the output.
fn flush(
    out: &mut Vec<AnyElement>,
    buf: &mut String,
    spans: &mut Vec<(Range<usize>, HighlightStyle)>,
    kind: Kind,
) {
    let text = std::mem::take(buf);
    let highlights = std::mem::take(spans);
    if text.trim().is_empty() {
        return;
    }

    let mut block = div()
        .font_family(FONT_PIXEL_ALT)
        .child(StyledText::new(text).with_highlights(highlights));

    block = match kind {
        Kind::Heading(size) => block
            .text_size(px(size as f32))
            .font_weight(FontWeight::EXTRA_BOLD)
            .text_color(rgb(TEXT_PRIMARY)),
        Kind::Code => block
            .text_size(px(14.))
            .text_color(rgb(TEXT_PRIMARY))
            .bg(rgb(BG_INPUT))
            .rounded(px(R_SM))
            .p(px(12.)),
        Kind::Quote => block
            .text_size(px(16.))
            .text_color(rgb(TEXT_MUTED))
            .border_l(px(2.))
            .border_color(rgb(BORDER))
            .pl(px(12.)),
        Kind::Item(level) => block
            .text_size(px(16.))
            .text_color(rgb(TEXT_SECONDARY))
            .pl(px(12. + 16. * level as f32)),
        Kind::Paragraph => block.text_size(px(16.)).text_color(rgb(TEXT_SECONDARY)),
    };

    out.push(block.into_any_element());
}

fn rule() -> AnyElement {
    div().h(px(1.)).w_full().bg(rgb(BORDER)).into_any_element()
}
