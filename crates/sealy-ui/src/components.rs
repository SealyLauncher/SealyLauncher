//! Shared visual building blocks for the Phase 1 shell.
//!
//! Deliberately no `SectionTitle` or `Subtitle` component: hierarchy is created
//! with spacing, surfaces, icons and a single display title per page.

use crate::kit::*;
use crate::theme;

/// Full-bleed layered background used behind every page.
pub fn background_stage() -> Div {
    div()
        .absolute()
        .top(px(0.0))
        .left(px(0.0))
        .size_full()
        .overflow_hidden()
        .bg(rgb(theme::CANVAS))
        .child(
            div()
                .absolute()
                .top(px(-190.0))
                .left(px(-150.0))
                .size(px(620.0))
                .rounded_full()
                .bg(rgb(theme::ACCENT_SOFT))
                .opacity(0.55),
        )
        .child(
            div()
                .absolute()
                .bottom(px(-260.0))
                .right(px(-160.0))
                .size(px(760.0))
                .rounded_full()
                .bg(rgb(theme::TEAL_DEEP))
                .opacity(0.4),
        )
        .child(
            div()
                .absolute()
                .top(px(70.0))
                .right(px(90.0))
                .size(px(320.0))
                .rounded_full()
                .bg(rgb(theme::ACCENT))
                .opacity(0.08),
        )
}

/// Translucent panel used for content surfaces.
pub fn glass_panel() -> Div {
    div()
        .bg(rgba(theme::GLASS))
        .border_1()
        .border_color(rgba(theme::GLASS_BORDER))
        .rounded(px(theme::RADIUS_LG))
}

/// Small status/metadata pill.
pub fn pill(text: impl Into<SharedString>) -> Div {
    let text: SharedString = text.into();
    div()
        .px_3()
        .py_1()
        .rounded_full()
        .bg(rgba(0xFFFFFF0D))
        .border_1()
        .border_color(rgba(0xFFFFFF14))
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::MUTED))
                .child(text),
        )
}

/// Primary action button body. Add `.id(...)` and `.on_click(...)` at the call site.
pub fn primary_button(label: impl Into<SharedString>) -> Div {
    let label: SharedString = label.into();
    div()
        .px_5()
        .py_3()
        .rounded(px(theme::RADIUS_MD))
        .bg(rgb(theme::ACCENT))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0x06121A))
                .child(label),
        )
}

/// Secondary action button body. Add `.id(...)` and `.on_click(...)` at the call site.
pub fn secondary_button(label: impl Into<SharedString>) -> Div {
    let label: SharedString = label.into();
    div()
        .px_4()
        .py_2()
        .rounded(px(theme::RADIUS_SM))
        .bg(rgba(0xFFFFFF0A))
        .border_1()
        .border_color(rgba(0xFFFFFF16))
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(theme::TEXT))
                .child(label),
        )
}

/// Soft icon tile used in empty states and method cards.
pub fn icon_tile(icon: IconName) -> Div {
    div()
        .w(px(44.0))
        .h(px(44.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_MD))
        .bg(rgba(0x4CC2FF14))
        .border_1()
        .border_color(rgba(0x4CC2FF26))
        .child(
            Icon::new(icon)
                .w(px(20.0))
                .h(px(20.0))
                .text_color(rgb(theme::ACCENT)),
        )
}

/// Hairline separator.
pub fn separator() -> Div {
    div().w_full().h(px(1.0)).bg(rgba(0xFFFFFF0A))
}

/// Centered content column.
pub fn content_column() -> Div {
    div()
        .w(px(theme::CONTENT_MAX_WIDTH))
        .flex()
        .flex_col()
        .gap_4()
}

/// Segmented option button body. Add `.id(...)` and `.on_click(...)`.
pub fn segment(label: impl Into<SharedString>, active: bool) -> Div {
    let label: SharedString = label.into();
    let base = div()
        .px_3()
        .py_2()
        .rounded(px(theme::RADIUS_SM))
        .border_1()
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if active {
                    rgb(theme::TEXT)
                } else {
                    rgb(theme::MUTED)
                })
                .child(label),
        );

    if active {
        base.bg(rgba(0x4CC2FF1F)).border_color(rgba(0x4CC2FF3D))
    } else {
        base.bg(rgba(0xFFFFFF08)).border_color(rgba(0xFFFFFF12))
    }
}
