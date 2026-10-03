use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, Context, FontWeight};
use i18n::t;

pub type Cx<'a> = Context<'a, LauncherUI>;

pub fn page_title(text: impl Into<gpui::SharedString>) -> AnyElement {
    let text = text.into();
    div()
        .font_family(FONT_PIXEL)
        .text_size(px(20.))
        .line_height(px(32.))
        .text_color(rgb(CTA))
        .child(text)
        .into_any_element()
}

pub const CONTENT_W: f32 = 720.;

pub fn panel() -> gpui::Div {
    div()
        .rounded(px(R_SM))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
}

pub fn page_header(icon: &'static str, title: impl Into<gpui::SharedString>) -> AnyElement {
    div()
        .h(px(72.))
        .px(px(24.))
        .flex()
        .items_center()
        .gap(px(12.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(crate::icons::ic(icon, 18., TEXT_MUTED))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(title.into()),
        )
        .into_any_element()
}

pub fn tabs(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let sid = ui.selected_server_id();
    let page = ui.page.clone();
    div()
        .absolute()
        .top(px(32.))
        .left(px(40.))
        .flex()
        .gap(px(8.))
        .child(tab(
            "tab-game",
            t("nav-game"),
            matches!(
                page,
                crate::state::Page::Servers | crate::state::Page::ServerDetail(_)
            ),
            move |this, cx| {
                this.page = sid
                    .map(crate::state::Page::ServerDetail)
                    .unwrap_or(crate::state::Page::Servers);
                cx.notify();
            },
            cx,
        ))
        .child(tab(
            "tab-optional",
            t("nav-optional"),
            matches!(page, crate::state::Page::ServerMods(_)),
            move |this, cx| {
                if let Some(id) = sid {
                    this.open_server(id);
                    this.page = crate::state::Page::ServerMods(id);
                    cx.notify();
                }
            },
            cx,
        ))
        // Your own content is a separate tab, and only where the build allows it.
        // The catalog used to be reached through a "suggest a mod" button on another
        // screen: that is not where anyone looks for a place to install a shader for
        // themselves.
        .when(
            sid.is_some_and(|id| ui.allow_personal_content.get(&id).copied().unwrap_or(false)),
            |d| {
                d.child(tab(
                    "tab-content",
                    t("nav-content"),
                    matches!(page, crate::state::Page::ServerModCatalog(_)),
                    move |this, cx| {
                        if let Some(id) = sid {
                            this.open_server(id);
                            this.content_mode = crate::state::ContentMode::Install;
                            this.page = crate::state::Page::ServerModCatalog(id);
                            cx.notify();
                        }
                    },
                    cx,
                ))
            },
        )
        .child(tab(
            "tab-settings",
            t("nav-settings"),
            matches!(page, crate::state::Page::ServerSettings(_)),
            move |this, cx| {
                if let Some(id) = sid {
                    this.open_server(id);
                    this.page = crate::state::Page::ServerSettings(id);
                    cx.notify();
                }
            },
            cx,
        ))
        .into_any_element()
}

fn tab(
    id: &'static str,
    label: impl Into<gpui::SharedString>,
    active: bool,
    on_click: impl Fn(&mut LauncherUI, &mut Cx) + 'static,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .id(id)
        .h(px(40.))
        .px(px(20.))
        .flex()
        .items_center()
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(if active {
            rgba(0xf3e7b3f0)
        } else {
            rgba(0x0f2036d8)
        })
        .border_1()
        .border_color(rgb(if active { CTA_HOV } else { BORDER }))
        .text_color(rgb(if active { ON_CTA } else { TEXT_SECONDARY }))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(16.))
        .font_weight(FontWeight::BOLD)
        .child(label.into())
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| on_click(this, cx)))
        .into_any_element()
}

pub fn initial(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into())
}

pub fn parse_hex(c: &str) -> u32 {
    u32::from_str_radix(c.trim_start_matches('#'), 16).unwrap_or(ACCENT)
}

/// "120 / 340 MB · 4.2 MB/s · 0:52 left" — the speed once it is known.
pub fn progress_label(s: &crate::state::SyncUiState) -> String {
    let total = s.total();
    if total == 0 {
        return String::new();
    }
    let size = crate::sync_text::megabytes(s.done(), total);
    match crate::sync_text::rate_label(&s.rate, s.done(), total) {
        Some(rate) => format!("{size} · {rate}"),
        None => size,
    }
}

/// Unix seconds as a short local date and time.
///
/// Fixed format rather than a locale one: GPUI has no date formatter, and the
/// alternative is an ICU dependency for six lines of text. Zero prints as a
/// dash — it is what an absent timestamp parses to, and "1 Jan 1970" reads as a
/// real date somebody then tries to explain.
pub fn short_date(unix_seconds: i64) -> String {
    if unix_seconds <= 0 {
        return "—".to_string();
    }
    chrono::DateTime::from_timestamp(unix_seconds, 0)
        .map(|utc| {
            chrono::DateTime::<chrono::Local>::from(utc)
                .format("%d.%m.%Y %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "—".to_string())
}

/// Unix seconds as a time of day, `14:47`.
pub fn short_time(unix_seconds: i64) -> String {
    stamp(unix_seconds, "%H:%M")
}

/// Unix seconds as a day, `27.09.2026`. Used for the separators in a chat.
pub fn short_day(unix_seconds: i64) -> String {
    stamp(unix_seconds, "%d.%m.%Y")
}

fn stamp(unix_seconds: i64, format: &str) -> String {
    if unix_seconds <= 0 {
        return "—".to_string();
    }
    chrono::DateTime::from_timestamp(unix_seconds, 0)
        .map(|utc| {
            chrono::DateTime::<chrono::Local>::from(utc)
                .format(format)
                .to_string()
        })
        .unwrap_or_else(|| "—".to_string())
}

/// Ctrl+V (Cmd+V on macOS) in one of the hand-made text fields: the clipboard
/// text, on one line. GPUI has no text input, so every field handles its own
/// keys — and only the JVM flags field had learned to paste.
pub fn pasted(event: &gpui::KeyDownEvent, cx: &mut gpui::App) -> Option<String> {
    let keystroke = &event.keystroke;
    if !(keystroke.modifiers.secondary() && keystroke.key == "v") {
        return None;
    }
    let text = cx.read_from_clipboard()?.text()?;
    Some(text.split_whitespace().collect::<Vec<_>>().join(" "))
}
