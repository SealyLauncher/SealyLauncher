//! Reusable Material 3 widgets expressed as GPUI elements.
//!
//! Every widget takes the derived scheme so a seed change repaints the whole
//! surface. Callers attach ids and handlers; these helpers only own shape,
//! colour roles and state layers.

use crate::kit::*;
use crate::m3::palette::{M3Scheme, mix, rgba_of};

/// Fully rounded shape.
pub const SHAPE_FULL: f32 = 999.0;
/// Large shape (cards, FAB).
pub const SHAPE_L: f32 = 16.0;
/// Medium shape (chips).
pub const SHAPE_M: f32 = 12.0;

/// Applies an M3 state layer over `base` using `ink`.
pub fn state(base: u32, ink: u32, alpha: f32) -> u32 {
    mix(base, ink, alpha)
}

/// Hover tint for a surface.
pub fn hover(base: u32, ink: u32) -> u32 {
    state(base, ink, 0.08)
}

/// Pressed tint for a surface.
pub fn pressed(base: u32, ink: u32) -> u32 {
    state(base, ink, 0.12)
}

fn icon(icon: IconName, size: f32, color: u32) -> Icon {
    Icon::new(icon)
        .w(px(size))
        .h(px(size))
        .text_color(rgb(color))
}

/// Navigation rail item: pill behind the icon, label underneath.
pub fn nav_item(
    name: IconName,
    label: impl Into<SharedString>,
    active: bool,
    scheme: &M3Scheme,
) -> Div {
    let label: SharedString = label.into();
    let fg = if active {
        scheme.on_secondary_container
    } else {
        scheme.on_surface_variant
    };
    let (pill, hover_pill) = if active {
        (scheme.secondary_container, scheme.secondary_container)
    } else {
        (
            scheme.surface,
            state(scheme.surface, scheme.on_surface, 0.08),
        )
    };

    div()
        .relative()
        .w(px(64.0))
        .h(px(56.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover_pill)))
        .child(
            div()
                .absolute()
                .top(px(4.0))
                .left(px(5.0))
                .w(px(54.0))
                .h(px(30.0))
                .rounded(px(SHAPE_FULL))
                .bg(rgb(pill)),
        )
        .child(icon(name, 20.0, fg))
        .child(
            div()
                .text_size(px(10.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(fg))
                .child(label),
        )
}

/// Filled button: primary container fill, `on_primary` ink.
pub fn filled_button(
    label: impl Into<SharedString>,
    name: Option<IconName>,
    scheme: &M3Scheme,
) -> Div {
    let label: SharedString = label.into();
    let base = scheme.primary;
    let ink = scheme.on_primary;

    let mut content = div().flex().items_center().gap(px(9.0)).child(
        div()
            .text_size(px(14.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgb(ink))
            .child(label),
    );
    if let Some(name) = name {
        content = content.child(icon(name, 16.0, ink));
    }

    div()
        .h(px(44.0))
        .px(px(24.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(SHAPE_FULL))
        .bg(rgb(base))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover(base, ink))))
        .child(content)
}

/// Outlined button: transparent fill, outline stroke, primary ink.
pub fn outlined_button(label: impl Into<SharedString>, scheme: &M3Scheme) -> Div {
    let label: SharedString = label.into();
    let ink = scheme.primary;

    div()
        .h(px(44.0))
        .px(px(24.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(SHAPE_FULL))
        .border_1()
        .border_color(rgb(scheme.outline))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(rgba_of(ink, 0.08))))
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(ink))
                .child(label),
        )
}

/// Round icon button (top app bar actions).
pub fn icon_button(name: IconName, scheme: &M3Scheme) -> Div {
    let ink = scheme.on_surface_variant;
    div()
        .w(px(40.0))
        .h(px(40.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(SHAPE_FULL))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover(scheme.surface, scheme.on_surface))))
        .child(icon(name, 18.0, ink))
}

/// Assist/filter chip. Selected chips use the secondary container role.
pub fn chip(label: impl Into<SharedString>, selected: bool, scheme: &M3Scheme) -> Div {
    let label: SharedString = label.into();
    let (bg, fg, border) = if selected {
        (scheme.secondary_container, scheme.on_secondary_container, 0)
    } else {
        (0, scheme.on_surface_variant, scheme.outline_variant)
    };

    let base = div()
        .h(px(32.0))
        .px(px(14.0))
        .flex()
        .items_center()
        .rounded(px(8.0))
        .cursor_pointer()
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(fg))
                .child(label),
        );

    if bg != 0 {
        base.bg(rgb(bg))
    } else {
        base.border_1().border_color(rgb(border))
    }
}

/// Surface container card.
pub fn card(scheme: &M3Scheme) -> Div {
    div()
        .p(px(16.0))
        .rounded(px(SHAPE_L))
        .bg(rgb(scheme.surface_container))
        .flex()
        .flex_col()
}

/// Tonal icon bubble used inside cards.
pub fn tonal_bubble(name: IconName, scheme: &M3Scheme) -> Div {
    div()
        .w(px(38.0))
        .h(px(38.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(SHAPE_FULL))
        .bg(rgb(scheme.primary_container))
        .child(icon(name, 18.0, scheme.on_primary_container))
}

/// Floating action button.
pub fn fab(name: IconName, scheme: &M3Scheme) -> Div {
    div()
        .w(px(56.0))
        .h(px(56.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(SHAPE_L))
        .bg(rgb(scheme.primary_container))
        .cursor_pointer()
        .shadow_lg()
        .hover(move |s| {
            s.bg(rgb(state(
                scheme.primary_container,
                scheme.on_primary_container,
                0.08,
            )))
        })
        .child(icon(name, 22.0, scheme.on_primary_container))
}

/// Linear progress indicator for `value` in `0..=1`.
pub fn linear_progress(value: f32, scheme: &M3Scheme) -> Div {
    let value = value.clamp(0.0, 1.0);
    div()
        .h(px(4.0))
        .w_full()
        .rounded(px(2.0))
        .bg(rgb(scheme.surface_container_highest))
        .child(
            div()
                .h_full()
                .w(relative(value))
                .rounded(px(2.0))
                .bg(rgb(scheme.primary)),
        )
}

/// One-shot ripple layer expanding from the centre of its parent.
///
/// `generation` changes on every press so GPUI restarts the animation.
pub fn ripple(generation: usize, ink: u32) -> Div {
    div()
        .absolute()
        .left(px(0.0))
        .top(px(0.0))
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .child(
            div()
                .rounded_full()
                .bg(rgba(rgba_of(ink, 0.22)))
                .with_animation(
                    ElementId::named_usize("m3-ripple", generation),
                    Animation::new(std::time::Duration::from_millis(460))
                        .with_easing(crate::motion::ease_out_cubic),
                    |el, t| el.size(px(28.0 + 420.0 * t)).opacity(1.0 - t),
                ),
        )
}
