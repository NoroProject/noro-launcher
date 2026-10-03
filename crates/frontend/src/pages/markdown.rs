// Over 150 lines: parsing, caching and drawing of one format, which share
// its block type and make sense only together.
//! Markdown news bodies into GPUI elements.
//!
//! Inline styles go on as ranges over a single `StyledText` rather than as
//! separate elements. Only that way does the paragraph wrap by words; split
//! into elements it breaks at every bold run.

use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, AnyElement, FontStyle, FontWeight, HighlightStyle, SharedString,
    StyledText,
};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Paragraph,
    Heading(u8),
    Code,
    Quote,
    Item(usize),
}

/// What a source parses into, kept between frames. Building elements from it
/// is cheap; running the parser over a long description sixty times a second
/// was not.
enum Block {
    Rule,
    Text {
        kind: Kind,
        text: SharedString,
        highlights: Vec<(Range<usize>, HighlightStyle)>,
    },
}

/// Recently shown sources. The news page and a mod page are the only users,
/// one text at a time, so a handful covers going back and forth.
const CACHED: usize = 8;

thread_local! {
    static CACHE: RefCell<Vec<(u64, Rc<Vec<Block>>)>> = const { RefCell::new(Vec::new()) };
}

fn cached(key: u64, parse: impl FnOnce() -> Vec<Block>) -> Rc<Vec<Block>> {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(pos) = cache.iter().position(|(k, _)| *k == key) {
            let entry = cache.remove(pos);
            let blocks = entry.1.clone();
            cache.push(entry);
            return blocks;
        }
        let blocks = Rc::new(parse());
        if cache.len() >= CACHED {
            cache.remove(0);
        }
        cache.push((key, blocks.clone()));
        blocks
    })
}

fn key(source: &str, salt: u8) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    salt.hash(&mut h);
    source.hash(&mut h);
    h.finish()
}

/// One element per paragraph, heading and list item.
pub fn render(source: &str) -> Vec<AnyElement> {
    cached(key(source, 0), || parse(source))
        .iter()
        .map(element)
        .collect()
}

/// For sources that need converting first (CurseForge sends HTML). The
/// conversion is cached along with the parse.
pub fn render_converted(source: &str, convert: fn(&str) -> String) -> Vec<AnyElement> {
    cached(key(source, 1), || parse(&convert(source)))
        .iter()
        .map(element)
        .collect()
}

fn parse(source: &str) -> Vec<Block> {
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
            Event::Rule => out.push(Block::Rule),
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
    // Counted as it grows: recounting the whole string after every event made
    // a long post quadratic.
    let mut chars = 0;
    for event in Parser::new(source) {
        match event {
            Event::Text(text) | Event::Code(text) => {
                chars += text.chars().count();
                out.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak | Event::End(TagEnd::Paragraph) => {
                chars += 1;
                out.push(' ');
            }
            _ => {}
        }
        if chars > limit {
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
    out: &mut Vec<Block>,
    buf: &mut String,
    spans: &mut Vec<(Range<usize>, HighlightStyle)>,
    kind: Kind,
) {
    let text = std::mem::take(buf);
    let highlights = std::mem::take(spans);
    if text.trim().is_empty() {
        return;
    }
    out.push(Block::Text {
        kind,
        text: text.into(),
        highlights,
    });
}

fn element(block: &Block) -> AnyElement {
    let (kind, text, highlights) = match block {
        Block::Rule => return div().h(px(1.)).w_full().bg(rgb(BORDER)).into_any_element(),
        Block::Text {
            kind,
            text,
            highlights,
        } => (*kind, text.clone(), highlights.clone()),
    };

    // The body font, not the pixel one: a mod description is paragraphs of running
    // text, and monospaced pixels make it read like a printed log. The pixel font
    // stays with headings and captions, where it is expected.
    let block = div()
        .font_family(FONT)
        .child(StyledText::new(text).with_highlights(highlights));

    match kind {
        Kind::Heading(size) => block
            .font_family(FONT_PIXEL_ALT)
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
            .text_size(px(14.))
            .text_color(rgb(TEXT_MUTED))
            .border_l(px(2.))
            .border_color(rgb(BORDER))
            .pl(px(12.)),
        Kind::Item(level) => block
            .text_size(px(14.))
            .text_color(rgb(TEXT_SECONDARY))
            .pl(px(12. + 16. * level as f32)),
        Kind::Paragraph => block.text_size(px(14.)).text_color(rgb(TEXT_SECONDARY)),
    }
    .into_any_element()
}
