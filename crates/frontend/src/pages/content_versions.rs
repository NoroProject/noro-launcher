// Over 150 lines: the picker is a dialog, a list and one row, and a row alone
// cannot say what happens when it is chosen.
//! Picking which version of a project to install.
//!
//! A picker rather than "install the latest". The newest file of a mod is
//! routinely published for a Minecraft version the build has not moved to yet,
//! and installing it quietly is how a launcher gets the reputation for breaking
//! games. Versions that do not fit are shown and refused, not hidden: "there is
//! nothing for this build yet" is an answer, an empty list is not.

use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::{ContentVersionInfo, MessageToBackend};
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use uuid::Uuid;

/// Nothing while no project is open, so the caller can pass it to `children`.
pub fn dialog(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> Option<AnyElement> {
    let (provider, project_id) = ui.content_picker.clone()?;
    let hit = ui
        .mod_catalog_hits
        .iter()
        .find(|h| h.project_id == project_id)
        .or(ui.mod_catalog_selected.as_ref())
        .cloned();
    let title = hit
        .as_ref()
        .map(|h| h.title.clone())
        .unwrap_or_else(|| project_id.clone());
    let icon_url = hit.as_ref().and_then(|h| h.icon_url.clone());

    let versions = ui
        .content_versions
        .get(&(provider.clone(), project_id.clone()))
        .cloned();

    // Подходящие вперёд и, если не попросили обратного, только они.
    let usable: Vec<&ContentVersionInfo> = versions
        .as_ref()
        .map(|list| {
            list.iter()
                .filter(|v| ui.content_versions_all || (v.compatible && v.downloadable))
                .collect()
        })
        .unwrap_or_default();
    let hidden = versions.as_ref().map(|l| l.len()).unwrap_or(0) - usable.len();

    let rows: Vec<AnyElement> = usable
        .iter()
        .map(|v| {
            row(
                server_id,
                &provider,
                &project_id,
                &title,
                icon_url.clone(),
                v,
                cx,
            )
        })
        .collect();

    Some(
        div()
            .id("version-picker-backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x000000aa))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.content_picker = None;
                cx.notify();
            }))
            .child(
                div()
                    .id("version-picker")
                    .occlude()
                    .on_click(|_, _, _| {})
                    .w(px(520.))
                    .max_h(px(480.))
                    .flex()
                    .flex_col()
                    .rounded(px(R_LG))
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(header(&title, cx))
                    .children(filter_note(ui, hidden, cx))
                    .child(
                        div()
                            .id("version-picker-scroll")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .p(px(8.))
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .children(rows)
                            .when(versions.is_none(), |d| d.child(loading()))
                            .when(versions.is_some() && usable.is_empty(), |d| {
                                d.child(nothing())
                            }),
                    ),
            )
            .into_any_element(),
    )
}

fn header(title: &str, cx: &mut Cx) -> AnyElement {
    div()
        .h(px(48.))
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(8.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .truncate()
                .child(format!("{} — {}", t("content-pick-version"), title)),
        )
        .child(
            div()
                .id("version-picker-close")
                .size(px(28.))
                .rounded(px(R_SM))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff10)))
                .child(ic("x", 14., TEXT_MUTED))
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.content_picker = None;
                    cx.notify();
                })),
        )
        .into_any_element()
}

/// Строка под шапкой: сколько версий скрыто и как их показать.
///
/// Без неё фильтр был бы обманом — список выглядел бы полным, а мода «под
/// другие версии» в нём просто нет.
fn filter_note(ui: &LauncherUI, hidden: usize, cx: &mut Cx) -> Option<AnyElement> {
    if hidden == 0 && !ui.content_versions_all {
        return None;
    }
    let showing_all = ui.content_versions_all;
    let mut args = i18n::FluentArgs::new();
    args.set("n", hidden.to_string());

    Some(
        div()
            .px(px(16.))
            .py(px(8.))
            .flex()
            .items_center()
            .gap(px(8.))
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .flex_1()
                    .text_size(px(11.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(if showing_all {
                        t("content-versions-showing-all")
                    } else {
                        i18n::t_args("content-versions-hidden", &args)
                    }),
            )
            .child(crate::components::segment(
                "versions-filter",
                None,
                if showing_all {
                    t("content-versions-only-fitting")
                } else {
                    t("content-versions-show-all")
                },
                showing_all,
                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.content_versions_all = !this.content_versions_all;
                    cx.notify();
                }),
            ))
            .into_any_element(),
    )
}

fn loading() -> AnyElement {
    centered(t("launcher-loading"), TEXT_MUTED)
}

fn nothing() -> AnyElement {
    centered(t("content-versions-none-fitting"), TEXT_MUTED)
}

fn centered(text: String, colour: u32) -> AnyElement {
    div()
        .py(px(48.))
        .flex()
        .justify_center()
        .text_size(px(12.))
        .text_color(rgb(colour))
        .child(text)
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn row(
    server_id: Uuid,
    provider: &str,
    project_id: &str,
    title: &str,
    icon_url: Option<String>,
    v: &ContentVersionInfo,
    cx: &mut Cx,
) -> AnyElement {
    let usable = v.compatible && v.downloadable;
    let (provider, project_id, title) = (
        provider.to_string(),
        project_id.to_string(),
        title.to_string(),
    );
    let version_id = v.id.clone();

    div()
        .id(SharedString::from(format!("version-{}", v.id)))
        .p(px(10.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(10.))
        .bg(rgba(0xffffff08))
        .when(usable, |d| {
            d.cursor_pointer().hover(|s| s.bg(rgba(0xffffff14)))
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(if usable { TEXT_PRIMARY } else { TEXT_MUTED }))
                        .truncate()
                        .child(if v.version_number.is_empty() {
                            v.name.clone()
                        } else {
                            v.version_number.clone()
                        }),
                )
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(rgb(reason_colour(v)))
                        .truncate()
                        .child(reason(v)),
                ),
        )
        .child(channel_chip(&v.channel))
        .when(usable, |d| {
            d.on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.content_busy = true;
                this.content_picker = None;
                this.content_error = None;
                this.backend.send(MessageToBackend::InstallPersonalContent {
                    server_id,
                    kind: this.content_kind,
                    provider: provider.clone(),
                    project_id: project_id.clone(),
                    version_id: version_id.clone(),
                    title: title.clone(),
                    icon_url: icon_url.clone(),
                });
                cx.notify();
            }))
        })
        .into_any_element()
}

fn reason(v: &ContentVersionInfo) -> String {
    if !v.downloadable {
        return t("content-version-blocked");
    }
    if !v.compatible {
        return t("content-version-incompatible");
    }
    // Compatible: say what it is for, which is what people actually scan.
    let mc = v.game_versions.join(", ");
    if v.loaders.is_empty() {
        mc
    } else {
        format!("{mc} · {}", v.loaders.join(", "))
    }
}

fn reason_colour(v: &ContentVersionInfo) -> u32 {
    if !v.downloadable {
        ERROR
    } else if !v.compatible {
        WARNING
    } else {
        TEXT_MUTED
    }
}

fn channel_chip(channel: &str) -> AnyElement {
    let colour = match channel {
        "release" => SUCCESS,
        "beta" => WARNING,
        _ => TEXT_MUTED,
    };
    div()
        .flex_shrink_0()
        .px(px(6.))
        .py(px(1.))
        .rounded(px(R_SM))
        .bg(rgba(0x00000040))
        .text_size(px(9.))
        .text_color(rgb(colour))
        .child(channel.to_uppercase())
        .into_any_element()
}
