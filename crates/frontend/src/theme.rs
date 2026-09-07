//! Design tokens — ATOM style: deep navy, cream call-to-action, magenta-pink
//! accent. Dark-first, and the only place colours are defined.
#![allow(dead_code)]

// ── Backgrounds ─────────────────────────────────────────────────────────────
pub const BG_WINDOW: u32 = 0x0d1b2e;
pub const BG_PANEL: u32 = 0x13233d;
pub const BG_CARD: u32 = 0x172a47;
pub const BG_CARD_HOV: u32 = 0x1f3556;
pub const BG_INPUT: u32 = 0x0f2036;
pub const SIDEBAR: u32 = 0x0b1626;
pub const BG_HEADER: u32 = 0x0b1626;
pub const OVERLAY: u32 = 0x081020;
/// Content area when the server has no background image.
pub const CONTENT_FALLBACK: u32 = 0x0a1626;

// ── Accents ─────────────────────────────────────────────────────────────────
/// Cream yellow; the primary call to action.
pub const CTA: u32 = 0xf3e7b3;
pub const CTA_HOV: u32 = 0xfbf0c4;
/// Text on top of the cream button.
pub const ON_CTA: u32 = 0x12233d;

/// Magenta pink: selection and highlights.
pub const ACCENT: u32 = 0xe85aa5;
pub const ACCENT_HOV: u32 = 0xf06fb4;
/// Secondary accent.
pub const BLUE: u32 = 0x7fb2ff;

// ── Status ──────────────────────────────────────────────────────────────────
pub const SUCCESS: u32 = 0x7ee0a4;
pub const WARNING: u32 = 0xf3c969;
pub const ERROR: u32 = 0xff6b8b;

// ── Text ────────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY: u32 = 0xdbe6ff;
pub const TEXT_SECONDARY: u32 = 0x9fb0d6;
// Same as the site. Anything darker misses 4.5:1 against the panel, and the
// pixel font used here is thin enough to need the margin.
pub const TEXT_MUTED: u32 = 0x8a9ac0;

// ── Borders ─────────────────────────────────────────────────────────────────
pub const BORDER: u32 = 0x223a55;
pub const BORDER_FOCUS: u32 = ACCENT;

// ── Fonts ───────────────────────────────────────────────────────────────────
pub const FONT: &str = "Inter";
/// Pixel font for the logo and large headings.
pub const FONT_PIXEL: &str = "Press Start 2P";
/// Pixel font for captions and section labels; more compact.
pub const FONT_PIXEL_ALT: &str = "Monocraft";

// ── Radii ───────────────────────────────────────────────────────────────────
pub const R_SM: f32 = 4.0;
pub const R_MD: f32 = 8.0;
pub const R_LG: f32 = 12.0;
