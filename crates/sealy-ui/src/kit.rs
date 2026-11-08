//! Thin facade over GPUI Kit.
//!
//! Pages must import GPUI types from this module, not directly from
//! `gpui_kit`, so a future backend swap stays localized.

pub use gpui_kit::assets::IconName;
pub use gpui_kit::component::Icon;
pub use gpui_kit::prelude::*;
pub use gpui_kit::{
    Animation, AnimationExt, AnyElement, Bounds, Context, Div, ElementId, FontWeight, IntoElement,
    Render, Role, SharedString, Window, WindowBounds, WindowOptions, div, px, relative, rgb, rgba,
    size,
};
