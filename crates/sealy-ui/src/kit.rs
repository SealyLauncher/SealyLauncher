//! Thin facade over GPUI Kit.
//!
//! Pages must import GPUI types from this module, not directly from
//! `gpui_kit`, so a future backend swap stays localized.

pub use gpui_kit::StyleRefinement;
pub use gpui_kit::assets::IconName;
pub use gpui_kit::base::slider::{SliderEvent, SliderState};
pub use gpui_kit::base::{Slider as BaseSlider, SliderIndicator, SliderThumb, SliderTrack};
pub use gpui_kit::component::Icon;
pub use gpui_kit::component::IndexPath;
pub use gpui_kit::component::Sizable;
pub use gpui_kit::component::WindowExt;
pub use gpui_kit::component::avatar::Avatar;
pub use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
pub use gpui_kit::component::dialog::Dialog;
pub use gpui_kit::component::input::Escape as InputEscape;
pub use gpui_kit::component::progress::Progress;
pub use gpui_kit::component::scroll::ScrollableElement;
pub use gpui_kit::component::select::{SearchableVec, Select, SelectEvent, SelectState};
pub use gpui_kit::component::sheet::Sheet;
pub use gpui_kit::component::spinner::Spinner;
pub use gpui_kit::component::switch::Switch;
pub use gpui_kit::component::tab::{Tab, TabBar};
pub use gpui_kit::component::{Disableable, RoleOverride, Selectable, StyledExt};
pub use gpui_kit::prelude::*;
pub use gpui_kit::{
    Animation, AnimationExt, AnyElement, App, Bounds, ClickEvent, ClipboardItem, Context,
    DefiniteLength, Div, ElementId, FocusHandle, FontWeight, Hsla, IntoElement, KeyDownEvent,
    Keystroke, ObjectFit, PathPromptOptions, Render, Rgba, Role, SharedString, Window,
    WindowBounds, WindowOptions, div, img, relative, rgb, rgba, size,
};
pub use gpui_kit::{AnyWindowHandle, Entity, Refineable, Subscription, WindowHandle};
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

/// Converts a packed RGB token into the Hsla that components expect.
pub fn token_hsla(color: u32) -> Hsla {
    gpui_kit::rgb(color).into()
}

/// Pixel helper that applies the configured interface scale.
pub fn px(value: impl Into<f64>) -> gpui_kit::Pixels {
    gpui_kit::px(value.into() as f32 * ui_scale())
}

/// Chain methods that the old `Div` buttons used, kept working on the
/// component `Button`.
pub trait ButtonChainExt: Sized {
    /// Accessibility label alias used across the pages.
    fn aria_label(self, label: impl Into<SharedString>) -> Self;
}

impl ButtonChainExt for Button {
    fn aria_label(self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label(label)
    }
}

/// Builds a stable element id from a helper's call site plus a discriminator.
pub(crate) fn button_id(location: &std::panic::Location<'_>, discriminator: &str) -> ElementId {
    ElementId::from(SharedString::from(format!(
        "{}:{}:{}:{discriminator}",
        location.file(),
        location.line(),
        location.column()
    )))
}
