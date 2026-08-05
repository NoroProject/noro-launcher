//! Кастомная оконная рамка: перетаскивание, переключатель языка, свернуть, закрыть.

use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, Context, MouseButton, MouseDownEvent, Pixels,
    Point, Size, WindowControlArea,
};
use i18n::Locale;

/// Верхняя панель окна. `compact` — тонкий вариант для основного интерфейса.
pub fn window_chrome(compact: bool, ui: &LauncherUI, cx: &mut Context<LauncherUI>) -> AnyElement {
    let active = ui.locale;
    div()
        .id("window-chrome")
        .h(px(if compact { 36. } else { 48. }))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .px(px(16.))
        .gap(px(4.))
        // Перетаскивание окна за рамку — но не за самый край.
        //
        // Панель занимает всю ширину и упирается в верхнюю кромку окна, а там
        // проходит системная зона ресайза. Когда `start_window_move()` висел на
        // всей площади, система успевала показать курсор ресайза, после чего
        // нажатие уводило окно в move: курсор мигал, размер не менялся, а
        // AppKit ругался «Window move completed without beginning».
        .on_mouse_down(MouseButton::Left, |event: &MouseDownEvent, window, _| {
            if !in_resize_edge(event.position, window.viewport_size()) {
                window.start_window_move();
            }
        })
        // Windows двигает окно сам, по ответу на WM_NCHITTEST, — там
        // `start_window_move()` не делает ничего, и шапка не таскалась вовсе.
        // Метку вешаем на пустую часть: накрыть ею всю панель нельзя, система
        // сочтёт кнопки частью заголовка и съест клики по ним.
        .child(
            div()
                .flex_1()
                .h_full()
                .window_control_area(WindowControlArea::Drag),
        )
        .children(Locale::ALL.map(|l| lang_pill(l, l == active, cx)))
        .child(div().w(px(8.)))
        .child(control("win-min", "minus", false))
        .child(control("win-close", "x", true))
        .into_any_element()
}

/// Ширина системной зоны ресайза по краям окна.
///
/// macOS ловит ресайз в нескольких пикселях от кромки; берём с запасом, иначе
/// попасть в неё мышью почти невозможно.
const RESIZE_EDGE: f32 = 6.;

fn in_resize_edge(position: Point<Pixels>, viewport: Size<Pixels>) -> bool {
    let edge = px(RESIZE_EDGE);
    position.y <= edge || position.x <= edge || position.x >= viewport.width - edge
}

fn lang_pill(locale: Locale, active: bool, cx: &mut Context<LauncherUI>) -> AnyElement {
    div()
        .id(locale.code())
        .px(px(8.))
        .h(px(24.))
        .flex()
        .items_center()
        .rounded(px(R_SM))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(14.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(if active { CTA } else { TEXT_MUTED }))
        .bg(if active {
            rgba((CTA << 8) | 0x18)
        } else {
            rgba(0x00000000)
        })
        .cursor_pointer()
        .hover(|d| d.bg(rgba(0xffffff10)))
        .child(locale.label())
        .on_click(cx.listener(move |this, _e, _w, cx| {
            this.set_locale(locale);
            cx.notify();
        }))
        .into_any_element()
}

fn control(id: &'static str, icon: &'static str, is_close: bool) -> AnyElement {
    div()
        .id(id)
        .size(px(26.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(R_SM))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(if is_close { ERROR } else { BG_CARD_HOV })))
        .child(ic(icon, 14., TEXT_SECONDARY))
        .on_click(move |_, window, cx| {
            if is_close {
                cx.quit();
            } else {
                window.minimize_window();
            }
        })
        .into_any_element()
}
