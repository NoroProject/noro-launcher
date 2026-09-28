// Over 150 lines: a catalogue card and the button on it. The button is the
// whole point of the card, and its two modes need the card's data.
//! One row of catalogue results.

use super::common::Cx;
use crate::icons::ic;
use crate::state::{ContentMode, LauncherUI};
use crate::theme::*;
use bridge::{CatalogHitInfo, MessageToBackend};
use gpui::{
    div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, ObjectFit,
    SharedString,
};
use i18n::t;
use uuid::Uuid;

/// Высота строки списка вместе с просветом под ней. Виртуальный список меряет
/// первую строку и раскладывает остальные по ней, поэтому карточки обязаны быть
/// одной высоты.
pub(super) const ROW_HEIGHT: f32 = 88.0;

pub(super) fn mod_card(
    ui: &LauncherUI,
    hit: CatalogHitInfo,
    server_id: Uuid,
    cx: &mut Cx,
) -> AnyElement {
    let hit_clone = hit.clone();
    let hit_for_req = hit.clone();
    let project_id_str = hit.project_id.clone();

    let action_btn = card_action(ui, server_id, &hit, hit_for_req, cx);

    // Обёртка нужна виртуальному списку: он считает высоту строки по первой и
    // отступов между элементами не знает, поэтому просвет живёт внутри строки.
    div()
        .h(px(ROW_HEIGHT))
        .pb(px(8.))
        .child(
            // Кликабельна вся карточка, а не колонка внутри неё: попасть в
            // строку легче, чем в её половину, и вложенный кликабельный блок
            // был лишним слоем в списке, который рисуется по многу раз.
            div()
                .id(SharedString::from(format!("mod-card-{project_id_str}")))
                .size_full()
                .px(px(14.))
                .rounded(px(R_MD))
                .bg(rgba(0xffffff08))
                .border_1()
                .border_color(rgb(BORDER))
                .flex()
                .items_center()
                .gap(px(14.))
                .cursor_pointer()
                .hover(|s| s.bg(rgba(0xffffff12)).border_color(rgb(BG_CARD_HOV)))
                .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.mod_catalog_selected = Some(hit_clone.clone());
                    this.mod_project = None;
                    this.mod_detail_gallery = false;
                    this.backend.send(MessageToBackend::RequestModProject {
                        provider: hit_clone.provider.clone(),
                        project_id: hit_clone.project_id.clone(),
                    });
                    cx.notify();
                }))
                .child(mod_avatar(ui, &hit.icon_url))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap(px(14.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap(px(2.))
                                .child(
                                    div().flex().items_center().gap(px(8.)).child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .font_family(FONT_PIXEL_ALT)
                                            .text_size(px(14.))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(TEXT_PRIMARY))
                                            .truncate()
                                            .child(hit.title.clone()),
                                    ), // Значок провайдера убран: он выбран
                                       // переключателем выше и одинаков у всей
                                       // выдачи — двадцать одинаковых плашек
                                       // не сообщали ничего.
                                )
                                .child(
                                    div()
                                        .truncate()
                                        .font_family(FONT_PIXEL_ALT)
                                        .text_size(px(11.))
                                        .text_color(rgb(TEXT_MUTED))
                                        .child(hit.description.clone()),
                                )
                                // Author and downloads are what people actually sort a
                                // catalogue by; the provider alone says nothing about
                                // whether a mod is the one everybody uses.
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(10.))
                                        .text_size(px(10.))
                                        .text_color(rgb(TEXT_MUTED))
                                        .children(hit.author.clone().map(|author| {
                                            div().child(format!("{} {author}", t("mods-by")))
                                        }))
                                        .when(hit.downloads > 0, |d| {
                                            d.child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(4.))
                                                    .child(ic("download", 10., TEXT_MUTED))
                                                    .child(compact(hit.downloads)),
                                            )
                                        }),
                                ),
                        ),
                )
                .child(action_btn),
        )
        .into_any_element()
}

/// The action on a catalogue card.
///
/// An icon, not a label. The word was the widest thing in the row — twenty
/// copies of «Предложить персоналу» read as a column of buttons with a list
/// hidden behind it, and which of the two things the button does was already
/// decided once, by the tab this screen was opened from.
fn card_action(
    ui: &LauncherUI,
    server_id: Uuid,
    hit: &CatalogHitInfo,
    hit_for_req: CatalogHitInfo,
    cx: &mut Cx,
) -> AnyElement {
    let project_id = hit.project_id.clone();

    // Already in the build: nothing to install and nothing to suggest.
    if super::mod_icon::is_mod_installed(ui, server_id, &hit.title) {
        return state_mark("check", SUCCESS);
    }

    match ui.content_mode {
        ContentMode::Suggest => {
            if ui.suggested_mods.contains(&project_id) {
                return state_mark("hourglass", WARNING);
            }
            action_button(
                format!("btn-req-{project_id}"),
                "send",
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    let build_id = this.server(&server_id).and_then(|s| s.current_build_id);
                    this.suggested_mods.insert(hit_for_req.project_id.clone());
                    this.backend.send(MessageToBackend::SuggestOptionalMod {
                        server_id,
                        build_id,
                        provider: hit_for_req.provider.clone(),
                        project_id: hit_for_req.project_id.clone(),
                        title: hit_for_req.title.clone(),
                        icon_url: hit_for_req.icon_url.clone(),
                        description: Some(hit_for_req.description.clone()),
                    });
                    cx.notify();
                }),
            )
        }
        ContentMode::Install => {
            if ui
                .personal_content
                .get(&server_id)
                .is_some_and(|items| items.iter().any(|i| i.content.project_id == project_id))
            {
                return state_mark("check", SUCCESS);
            }
            let provider = hit.provider.clone();
            let pid = project_id.clone();
            action_button(
                format!("btn-install-{project_id}"),
                "plus",
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    // Opens the version list rather than installing at once:
                    // the newest file is often built for a Minecraft version
                    // this build has not reached.
                    this.content_error = None;
                    this.content_picker = Some((provider.clone(), pid.clone()));
                    this.content_versions
                        .remove(&(provider.clone(), pid.clone()));
                    this.backend.send(MessageToBackend::RequestContentVersions {
                        provider: provider.clone(),
                        project_id: pid.clone(),
                        server_id,
                    });
                    cx.notify();
                }),
            )
        }
    }
}

/// A round button carrying only an icon.
fn action_button(
    id: String,
    icon: &'static str,
    on_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(SharedString::from(id))
        // Клик по кнопке не должен открывать страницу мода: она лежит внутри
        // карточки, а та кликабельна целиком.
        .occlude()
        .size(px(36.))
        .flex_shrink_0()
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .group("content-action")
        .bg(rgba((CTA << 8) | 0x1a))
        .border_1()
        .border_color(rgb(BORDER))
        .hover(|d| d.bg(rgb(CTA)).border_color(rgb(CTA_HOV)))
        // Цвет значка приходится задавать самому значку: `text_color`
        // родителя svg не наследует, и попытка перекрасить кнопку целиком
        // оставляла на ней пустой квадрат. `group_hover` смотрит на наведение
        // родителя — на кремовом фоне значок становится тёмным.
        .child(
            gpui::svg()
                .path(format!("icons/{icon}.svg"))
                .size(px(16.))
                .text_color(rgb(CTA))
                .group_hover("content-action", |s| s.text_color(rgb(ON_CTA))),
        )
        .on_click(on_click)
        .into_any_element()
}

/// Nothing to press: it is in already, or the request is with staff.
fn state_mark(icon: &'static str, colour: u32) -> AnyElement {
    div()
        .size(px(36.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .child(ic(icon, 16., colour))
        .into_any_element()
}

pub(super) fn mod_avatar(ui: &LauncherUI, icon_url: &Option<String>) -> AnyElement {
    let outer = div()
        .size(px(44.))
        .rounded(px(R_SM))
        .overflow_hidden()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center();

    if let Some(ref url) = icon_url {
        if let Some(img_data) = ui.optional_mod_icons.get(url).cloned() {
            return outer
                .child(img(img_data).size_full().object_fit(ObjectFit::Cover))
                .into_any_element();
        }
    }
    outer
        .bg(rgba(0xffffff15))
        .border_1()
        .border_color(rgb(BORDER))
        .child(ic("box", 20., TEXT_MUTED))
        .into_any_element()
}

/// `1_240_000` → `1.2M`. A download count is read as an order of magnitude, and
/// seven digits in a card that is mostly text is just noise.
pub(super) fn compact_downloads(n: u64) -> String {
    compact(n)
}

fn compact(n: u64) -> String {
    match n {
        0..=999 => n.to_string(),
        1_000..=999_999 => format!("{:.1}K", n as f64 / 1_000.0),
        _ => format!("{:.1}M", n as f64 / 1_000_000.0),
    }
}

/// The action as the mod's own page shows it: icon plus words.
///
/// A card has room for a glyph and twenty rows of them; a page has one action
/// and space to name it.
pub(super) fn detail_action(
    ui: &LauncherUI,
    server_id: Uuid,
    hit: &CatalogHitInfo,
    cx: &mut Cx,
) -> AnyElement {
    let project_id = hit.project_id.clone();
    let installed = super::mod_icon::is_mod_installed(ui, server_id, &hit.title)
        || ui
            .personal_content
            .get(&server_id)
            .is_some_and(|items| items.iter().any(|i| i.content.project_id == project_id));

    if installed {
        return wide_state("check", t("mods-installed"), SUCCESS);
    }
    if ui.content_mode == ContentMode::Suggest && ui.suggested_mods.contains(&project_id) {
        return wide_state("hourglass", t("content-pending"), WARNING);
    }

    let hit = hit.clone();
    let suggest = ui.content_mode == ContentMode::Suggest;
    let (icon, label) = if suggest {
        ("send", t("content-mode-suggest"))
    } else {
        ("plus", t("content-install"))
    };

    div()
        .id("mod-detail-action")
        .flex_shrink_0()
        .h(px(40.))
        .px(px(18.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .bg(rgb(CTA))
        .text_color(rgb(ON_CTA))
        .hover(|d| d.bg(rgb(CTA_HOV)))
        .child(ic(icon, 15., ON_CTA))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(FontWeight::BOLD)
                .child(label),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            if suggest {
                let build_id = this.server(&server_id).and_then(|s| s.current_build_id);
                this.suggested_mods.insert(hit.project_id.clone());
                this.backend.send(MessageToBackend::SuggestOptionalMod {
                    server_id,
                    build_id,
                    provider: hit.provider.clone(),
                    project_id: hit.project_id.clone(),
                    title: hit.title.clone(),
                    icon_url: hit.icon_url.clone(),
                    description: Some(hit.description.clone()),
                });
            } else {
                this.content_error = None;
                this.content_picker = Some((hit.provider.clone(), hit.project_id.clone()));
                this.content_versions
                    .remove(&(hit.provider.clone(), hit.project_id.clone()));
                this.backend.send(MessageToBackend::RequestContentVersions {
                    provider: hit.provider.clone(),
                    project_id: hit.project_id.clone(),
                    server_id,
                });
            }
            cx.notify();
        }))
        .into_any_element()
}

/// Nothing to press, said in words.
fn wide_state(icon: &'static str, label: String, colour: u32) -> AnyElement {
    div()
        .flex_shrink_0()
        .h(px(40.))
        .px(px(16.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(8.))
        .bg(rgba(0x00000040))
        .border_1()
        .border_color(rgb(colour))
        .child(ic(icon, 15., colour))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(colour))
                .child(label),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::compact;

    #[test]
    fn small_counts_stay_exact() {
        assert_eq!(compact(0), "0");
        assert_eq!(compact(999), "999");
    }

    #[test]
    fn thousands_and_millions_are_shortened() {
        assert_eq!(compact(1_500), "1.5K");
        assert_eq!(compact(1_240_000), "1.2M");
    }
}
