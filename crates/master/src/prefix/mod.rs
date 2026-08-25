//! Префиксы ролей картинкой: плашка вместо звёздочки.
//!
//! Мастер рисует по плашке на роль и складывает их в ресурспак с bitmap-шрифтом.
//! В игре плашка — один символ в своём шрифте `noro:prefix`; поле шрифта
//! проставляют форки TAB и StyledChat, поэтому за пределами чата и таба глиф не
//! появляется нигде.
//!
//! Подробности и принятые решения — в `docs/prefix-pack-plan.md`.

mod badge;
mod font;
mod pack;
mod store;

#[cfg(test)]
mod tests;

pub use pack::{build, Built, Role};
pub use store::{current, rebuild, Current, Glyph};

/// Высота нарисованной плашки. По ней проверяются загруженные вручную: чтобы
/// сесть в строку чата, картинка должна быть кратна ей.
pub fn badge_height() -> usize {
    badge::height()
}

/// Плашка картинкой — для предпросмотра в админке. Та же, что уедет в игру.
pub fn badge_png(text: &str, color: &str) -> anyhow::Result<Vec<u8>> {
    badge::png(text, color)
}

/// Плашка вместе с ником: так предпросмотр показывает целую строку чата.
pub fn badge_line(text: &str, color: &str, name: &str) -> anyhow::Result<Vec<u8>> {
    badge::line(text, color, name)
}

/// Первая кодовая точка приватной области. Дальше по порядку, роль за ролью.
///
/// Приватная область именно для того и выделена: свои символы туда класть можно,
/// и ни один язык на них не претендует.
pub const FIRST_CODEPOINT: u32 = 0xE000;

/// Символ роли по её порядковому номеру в паке.
pub fn codepoint(index: usize) -> char {
    char::from_u32(FIRST_CODEPOINT + index as u32).unwrap_or('\u{E000}')
}
