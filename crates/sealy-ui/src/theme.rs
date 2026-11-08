//! Phase 1 design tokens.
//!
//! Colors are packed RGB(A) values so they remain plain constants; call
//! `rgb(...)` / `rgba(...)` at the element layer.

pub const CANVAS: u32 = 0x0B0D12;
pub const CANVAS_ELEVATED: u32 = 0x10141B;
pub const SURFACE: u32 = 0x151A23;
pub const SURFACE_RAISED: u32 = 0x1B2230;
pub const BORDER: u32 = 0x253041;
pub const BORDER_SOFT: u32 = 0x1B2330;
pub const TEXT: u32 = 0xF4F7FB;
pub const MUTED: u32 = 0x8A94A6;
pub const FAINT: u32 = 0x5A6474;
pub const ACCENT: u32 = 0x4CC2FF;
pub const ACCENT_SOFT: u32 = 0x163A4A;
pub const ACCENT_GLOW: u32 = 0x1B6A8A;
pub const TEAL_DEEP: u32 = 0x0E4B4A;
pub const DANGER: u32 = 0xFF6B6B;
pub const SUCCESS: u32 = 0x6BE3A2;

/// Translucent panel fill (RGBA).
pub const GLASS: u32 = 0x12151CCC;
/// Translucent panel border (RGBA).
pub const GLASS_BORDER: u32 = 0xFFFFFF14;

pub const RADIUS_SM: f32 = 10.0;
pub const RADIUS_MD: f32 = 14.0;
pub const RADIUS_LG: f32 = 20.0;
pub const RADIUS_XL: f32 = 28.0;

pub const NAV_WIDTH: f32 = 76.0;
pub const PAGE_PADDING: f32 = 40.0;
pub const CONTENT_MAX_WIDTH: f32 = 760.0;
