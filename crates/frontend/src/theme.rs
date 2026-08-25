//! Дизайн-токены лаунчера — стиль ATOM: тёмно-синий космос, кремовые CTA,
//! розово-маджентовый акцент-кристалл. Dark-first, единственный источник цветов.
#![allow(dead_code)]

// ── Фоны ────────────────────────────────────────────────────────────────────
pub const BG_WINDOW: u32 = 0x0d1b2e; // глубокий navy
pub const BG_PANEL: u32 = 0x13233d;
pub const BG_CARD: u32 = 0x172a47;
pub const BG_CARD_HOV: u32 = 0x1f3556;
pub const BG_INPUT: u32 = 0x0f2036;
pub const SIDEBAR: u32 = 0x0b1626;
pub const BG_HEADER: u32 = 0x0b1626;
pub const OVERLAY: u32 = 0x081020;
/// Фон контентной области, когда нет фоновой картинки сервера.
pub const CONTENT_FALLBACK: u32 = 0x0a1626;

// ── Акценты ───────────────────────────────────────────────────────────────────
/// Кремово-жёлтый CTA (как кнопка Sign in на концепте).
pub const CTA: u32 = 0xf3e7b3;
pub const CTA_HOV: u32 = 0xfbf0c4;
/// Текст поверх кремовой кнопки.
pub const ON_CTA: u32 = 0x12233d;

/// Розово-маджентовый акцент-кристалл (выделение, подсветка).
pub const ACCENT: u32 = 0xe85aa5;
pub const ACCENT_HOV: u32 = 0xf06fb4;
/// Голубой вторичный акцент.
pub const BLUE: u32 = 0x7fb2ff;

// ── Статусы ───────────────────────────────────────────────────────────────────
pub const SUCCESS: u32 = 0x7ee0a4;
pub const WARNING: u32 = 0xf3c969;
pub const ERROR: u32 = 0xff6b8b;

// ── Текст ───────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY: u32 = 0xdbe6ff;
pub const TEXT_SECONDARY: u32 = 0x9fb0d6;
// Тот же цвет, что и на сайте: прежний 0x5a6b91 не дотягивал до 4.5:1 на
// панели, а шрифт здесь тот же тонкий пиксельный.
pub const TEXT_MUTED: u32 = 0x8a9ac0;

// ── Границы ───────────────────────────────────────────────────────────────────
pub const BORDER: u32 = 0x223a55;
pub const BORDER_FOCUS: u32 = ACCENT;

// ── Шрифты ───────────────────────────────────────────────────────────────────
/// Основной текстовый шрифт.
pub const FONT: &str = "Inter";
/// Пиксельный шрифт для логотипа/крупных заголовков.
pub const FONT_PIXEL: &str = "Press Start 2P";
/// Пиксельный шрифт для подписей/секций (компактнее).
pub const FONT_PIXEL_ALT: &str = "Monocraft";

// ── Скругления / радиусы ───────────────────────────────────────────────────────
pub const R_SM: f32 = 4.0;
pub const R_MD: f32 = 8.0;
pub const R_LG: f32 = 12.0;
