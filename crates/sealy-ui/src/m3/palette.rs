//! Material 3 colour roles derived from a single seed colour.

use material_colors::{
    color::Rgb,
    dynamic_color::{DynamicScheme, Variant},
};

/// M3 colour roles used by the launcher, packed as `0xRRGGBB`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M3Scheme {
    /// Whether this is the dark scheme.
    pub dark: bool,
    /// Seed colour the scheme was derived from.
    pub seed: u32,

    /// Primary role.
    pub primary: u32,
    /// Content on primary.
    pub on_primary: u32,
    /// Primary container.
    pub primary_container: u32,
    /// Content on primary container.
    pub on_primary_container: u32,

    /// Secondary container.
    pub secondary_container: u32,
    /// Content on secondary container.
    pub on_secondary_container: u32,

    /// Tertiary.
    pub tertiary: u32,
    /// Content on tertiary.
    pub on_tertiary: u32,
    /// Tertiary container.
    pub tertiary_container: u32,
    /// Content on tertiary container.
    pub on_tertiary_container: u32,

    /// Error.
    pub error: u32,
    /// Content on error.
    pub on_error: u32,

    /// Base surface.
    pub surface: u32,
    /// Content on surface.
    pub on_surface: u32,
    /// Surface variant.
    pub surface_variant: u32,
    /// Content on surface variant.
    pub on_surface_variant: u32,

    /// Lowest container tone.
    pub surface_container_lowest: u32,
    /// Low container tone.
    pub surface_container_low: u32,
    /// Container tone.
    pub surface_container: u32,
    /// High container tone.
    pub surface_container_high: u32,
    /// Highest container tone.
    pub surface_container_highest: u32,

    /// Outline.
    pub outline: u32,
    /// Outline variant.
    pub outline_variant: u32,
    /// Surface tint.
    pub surface_tint: u32,
}

fn to_u32(c: Rgb) -> u32 {
    (u32::from(c.red) << 16) | (u32::from(c.green) << 8) | u32::from(c.blue)
}

/// Derives an M3 scheme from `seed`.
pub fn scheme(seed: u32, dark: bool) -> M3Scheme {
    let source = Rgb::from_u32(seed);
    let s = DynamicScheme::by_variant(source, &Variant::TonalSpot, dark, None);

    M3Scheme {
        dark,
        seed,
        primary: to_u32(s.primary()),
        on_primary: to_u32(s.on_primary()),
        primary_container: to_u32(s.primary_container()),
        on_primary_container: to_u32(s.on_primary_container()),
        secondary_container: to_u32(s.secondary_container()),
        on_secondary_container: to_u32(s.on_secondary_container()),
        tertiary: to_u32(s.tertiary()),
        on_tertiary: to_u32(s.on_tertiary()),
        tertiary_container: to_u32(s.tertiary_container()),
        on_tertiary_container: to_u32(s.on_tertiary_container()),
        error: to_u32(s.error()),
        on_error: to_u32(s.on_error()),
        surface: to_u32(s.surface()),
        on_surface: to_u32(s.on_surface()),
        surface_variant: to_u32(s.surface_variant()),
        on_surface_variant: to_u32(s.on_surface_variant()),
        surface_container_lowest: to_u32(s.surface_container_lowest()),
        surface_container_low: to_u32(s.surface_container_low()),
        surface_container: to_u32(s.surface_container()),
        surface_container_high: to_u32(s.surface_container_high()),
        surface_container_highest: to_u32(s.surface_container_highest()),
        outline: to_u32(s.outline()),
        outline_variant: to_u32(s.outline_variant()),
        surface_tint: to_u32(s.surface_tint()),
    }
}

/// Linear blend of two packed colours; `t = 0` returns `base`.
pub fn mix(base: u32, overlay: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |shift: u32| -> u32 {
        let a = (base >> shift) & 0xFF;
        let b = (overlay >> shift) & 0xFF;
        (a as f32 + (b as f32 - a as f32) * t).round() as u32 & 0xFF
    };
    (lerp(16) << 16) | (lerp(8) << 8) | lerp(0)
}

/// Packs a colour with alpha into GPUI's `0xRRGGBBAA` form.
pub fn rgba_of(color: u32, alpha: f32) -> u32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u32;
    (color << 8) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_and_light_differ() {
        let dark = scheme(0x6750A4, true);
        let light = scheme(0x6750A4, false);
        assert_ne!(dark.surface, light.surface);
        assert_ne!(dark.on_surface, light.on_surface);
        assert!(dark.dark);
        assert!(!light.dark);
    }

    #[test]
    fn mix_endpoints() {
        assert_eq!(mix(0x000000, 0xFFFFFF, 0.0), 0x000000);
        assert_eq!(mix(0x000000, 0xFFFFFF, 1.0), 0xFFFFFF);
    }
}
