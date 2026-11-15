//! Thin facade over GPUI Kit.
//!
//! Pages must import GPUI types from this module, not directly from
//! `gpui_kit`, so a future backend swap stays localized.

pub use gpui_kit::assets::IconName;
pub use gpui_kit::base::slider::{SliderEvent, SliderState};
pub use gpui_kit::base::{Slider as BaseSlider, SliderIndicator, SliderThumb, SliderTrack};
pub use gpui_kit::component::Icon;
pub use gpui_kit::prelude::*;
pub use gpui_kit::{
    Animation, AnimationExt, AnyElement, App, Bounds, ClickEvent, ClipboardItem, Context, Div,
    ElementId, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Keystroke, ObjectFit,
    PathPromptOptions, Render, Rgba, Role, SharedString, Window, WindowBounds, WindowOptions, div,
    img, relative, rgb, rgba, size,
};
pub use gpui_kit::{AnyWindowHandle, Entity, Subscription, WindowHandle};
pub use gpui_kit::{KeyBinding, Menu, MenuItem, NoAction, OsAction, SystemMenuType};

/// Global interface scale (1.0 = default).
static UI_SCALE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3F80_0000);

/// Current interface scale.
pub fn ui_scale() -> f32 {
    f32::from_bits(UI_SCALE.load(std::sync::atomic::Ordering::Relaxed))
}

/// Updates the interface scale, clamped to a usable range.
pub fn set_ui_scale(scale: f32) {
    let clamped = scale.clamp(0.7, 1.6);
    UI_SCALE.store(clamped.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

/// Pixel helper that applies the configured interface scale.
pub fn px(value: impl Into<f64>) -> gpui_kit::Pixels {
    gpui_kit::px(value.into() as f32 * ui_scale())
}
