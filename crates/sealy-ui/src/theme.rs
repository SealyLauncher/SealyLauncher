//! Theme tokens.
//!
//! Every colour the UI draws comes from [`ThemeTokens`]. Presets share one set
//! of token names so a theme switch is a pure value swap, and an OEM build can
//! override `accent`/`accent_text` without touching anything else.

use sealy_core::config::ThemeName;

/// A complete colour set for one theme preset.
///
/// Colours are packed `0xRRGGBB`; call [`rgb`](gpui_kit::rgb) at the element
/// layer so alpha stays explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeTokens {
    /// Window background.
    pub bg: u32,
    /// Raised surface used by tiles, bars and cards.
    pub surface: u32,
    /// Structural border.
    pub line: u32,
    /// Divider inside a surface.
    pub line_soft: u32,
    /// Primary text.
    pub text: u32,
    /// Secondary text.
    pub muted: u32,
    /// Tertiary text (timestamps, indices).
    pub faint: u32,
    /// Inverted block fill.
    pub block: u32,
    /// Text on [`block`](Self::block).
    pub block_text: u32,
    /// Single emphasis colour, reserved for the primary action.
    pub accent: u32,
    /// Text on [`accent`](Self::accent).
    pub accent_text: u32,
    /// Destructive colour.
    pub danger: u32,
}

impl ThemeTokens {
    /// Resolves a preset to its token values.
    pub fn preset(name: ThemeName) -> Self {
        match name {
            ThemeName::Paper => Self {
                bg: 0xF4F2EE,
                surface: 0xFFFFFF,
                line: 0xDCD8CF,
                line_soft: 0xE8E4DB,
                text: 0x101010,
                muted: 0x6F6A61,
                faint: 0xA8A29A,
                block: 0x0D0D0D,
                block_text: 0xF4F2EE,
                accent: 0x0D0D0D,
                accent_text: 0xF4F2EE,
                danger: 0xC42B1C,
            },
            ThemeName::Ink => Self {
                bg: 0x14161A,
                surface: 0x1B1F26,
                line: 0x2A3038,
                line_soft: 0x232830,
                text: 0xEDF1F6,
                muted: 0x8B94A3,
                faint: 0x5C6472,
                block: 0xF2F4F7,
                block_text: 0x101318,
                accent: 0xF2F4F7,
                accent_text: 0x101318,
                danger: 0xC42B1C,
            },
            ThemeName::Clay => Self {
                bg: 0xF3ECE3,
                surface: 0xFFFDF9,
                line: 0xE0D4C4,
                line_soft: 0xECE2D5,
                text: 0x1B1512,
                muted: 0x766A5E,
                faint: 0xAB9D8E,
                block: 0x2B1C14,
                block_text: 0xF7F0E7,
                accent: 0xB4552A,
                accent_text: 0xFFF8F2,
                danger: 0xC42B1C,
            },
            ThemeName::Celadon => Self {
                bg: 0xEEF2EF,
                surface: 0xFFFFFF,
                line: 0xD3DCD6,
                line_soft: 0xE3EAE5,
                text: 0x10201B,
                muted: 0x5F6F68,
                faint: 0x9AAAA2,
                block: 0x0F3D33,
                block_text: 0xEEF5F1,
                accent: 0x0F766E,
                accent_text: 0xF2FBF8,
                danger: 0xC42B1C,
            },
        }
    }
}

/// Packs a colour with an alpha byte for [`rgba`](gpui_kit::rgba).
pub fn alpha(color: u32, a: u8) -> u32 {
    (color << 8) | u32::from(a)
}

/// Window size used by the shell.
pub const WINDOW_WIDTH: f32 = 1120.0;
/// Window height used by the shell.
pub const WINDOW_HEIGHT: f32 = 700.0;
/// Minimum window size: the tile grid needs 950 px plus two page insets.
pub const MIN_WIDTH: f32 = 1030.0;
/// Minimum window height.
pub const MIN_HEIGHT: f32 = 640.0;

/// Outer window radius.
pub const RADIUS_WINDOW: f32 = 6.0;
/// Tile and control radius.
pub const RADIUS_TILE: f32 = 2.0;

/// Title band height.
pub const HEAD_HEIGHT: f32 = 82.0;
/// Persistent action bar height.
pub const ACTION_BAR_HEIGHT: f32 = 84.0;
/// Small tile edge.
pub const TILE_SM: f32 = 84.0;
/// Gap between tiles.
pub const TILE_GAP: f32 = 10.0;
/// Medium tile edge (two small tiles plus one gap).
pub const TILE_MD: f32 = TILE_SM * 2.0 + TILE_GAP;
/// Horizontal page padding.
pub const PAGE_PADDING: f32 = 32.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_defines_contrast_pairs() {
        for name in ThemeName::ALL {
            let t = ThemeTokens::preset(name);
            assert_ne!(t.text, t.bg, "{name:?} text must differ from bg");
            assert_ne!(t.accent, t.accent_text, "{name:?} accent pair");
            assert_ne!(t.block, t.block_text, "{name:?} block pair");
        }
    }

    #[test]
    fn presets_are_distinct() {
        let paper = ThemeTokens::preset(ThemeName::Paper);
        let ink = ThemeTokens::preset(ThemeName::Ink);
        assert_ne!(paper.bg, ink.bg);
        assert_ne!(paper.text, ink.text);
    }
}
