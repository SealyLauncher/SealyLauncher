//! Thin facade over GPUI Kit.
//!
//! Pages must import GPUI types from this module, not directly from
//! `gpui_kit`, so a future backend swap stays localized.

pub use gpui_kit::assets::IconName;
pub use gpui_kit::component::Icon;
pub use gpui_kit::prelude::*;
pub use gpui_kit::{
    Animation, AnimationExt, AnyElement, App, Bounds, ClickEvent, ClipboardItem, Context, Div,
    ElementId, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Keystroke, ObjectFit,
    PathPromptOptions, Render, Rgba, Role, SharedString, Window, WindowBounds, WindowOptions, div,
    img, px, relative, rgb, rgba, size,
};
