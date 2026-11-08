//! Shared building blocks for the editorial shell.
//!
//! Components only own shape and colour; interaction ids and handlers stay in
//! the pages so the same tile can navigate, launch, or do nothing.

use crate::kit::*;
use crate::theme::{self, ThemeTokens};

/// Uppercase monospace label used for indices, dates and durations.
pub fn mono(text: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let text: SharedString = text.into();
    div()
        .text_size(px(10.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(t.faint))
        .child(text)
}

/// Square index badge (`01`) used before a page title.
pub fn index_badge(index: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let index: SharedString = index.into();
    div()
        .w(px(26.0))
        .h(px(18.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.block))
        .child(
            div()
                .text_size(px(10.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(t.block_text))
                .child(index),
        )
}

/// Page title band: index badge plus the single large title.
pub fn head_band(index: &str, title: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let title: SharedString = title.into();
    div()
        .h(px(theme::HEAD_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(14.0))
        .px(px(theme::PAGE_PADDING))
        .bg(rgb(t.surface))
        .border_b_1()
        .border_color(rgb(t.line))
        .child(index_badge(index, t))
        .child(
            div()
                .text_size(px(42.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(title),
        )
}

/// Base tile surface.
pub fn tile(t: &ThemeTokens) -> Div {
    div()
        .relative()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.surface))
        .border_1()
        .border_color(rgb(t.line))
        .overflow_hidden()
        .cursor_pointer()
        .hover(move |s| s.border_color(rgb(t.text)))
}

/// Small square tile: icon only.
pub fn small_tile(icon: IconName, active: bool, t: &ThemeTokens) -> Div {
    let base = tile(t)
        .w(px(theme::TILE_SM))
        .h(px(theme::TILE_SM))
        .flex()
        .items_center()
        .justify_center();

    if active {
        base.bg(rgb(t.block)).border_color(rgb(t.block)).child(
            Icon::new(icon)
                .w(px(22.0))
                .h(px(22.0))
                .text_color(rgb(t.block_text)),
        )
    } else {
        base.child(
            Icon::new(icon)
                .w(px(22.0))
                .h(px(22.0))
                .text_color(rgb(t.text)),
        )
    }
}

/// Medium tile: icon above a label.
pub fn medium_tile(icon: IconName, label: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    tile(t)
        .w(px(theme::TILE_MD))
        .h(px(theme::TILE_MD))
        .flex()
        .flex_col()
        .justify_end()
        .gap(px(10.0))
        .p(px(14.0))
        .child(
            Icon::new(icon)
                .w(px(24.0))
                .h(px(24.0))
                .text_color(rgb(t.text)),
        )
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(label),
        )
}

/// Wide tile: framed icon, label, trailing arrow.
pub fn wide_tile(icon: IconName, label: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    tile(t)
        .w(px(theme::TILE_MD * 2.0 + theme::TILE_GAP))
        .h(px(theme::TILE_MD))
        .flex()
        .items_center()
        .gap(px(16.0))
        .px(px(20.0))
        .child(
            div()
                .w(px(54.0))
                .h(px(54.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.line))
                .child(
                    Icon::new(icon)
                        .w(px(26.0))
                        .h(px(26.0))
                        .text_color(rgb(t.text)),
                ),
        )
        .child(
            div()
                .text_size(px(17.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(label),
        )
        .child(div().flex_1())
        .child(
            Icon::new(IconName::ArrowRight)
                .w(px(15.0))
                .h(px(15.0))
                .text_color(rgb(t.faint)),
        )
}

/// Inverted block used for the notices tile.
pub fn block_tile(t: &ThemeTokens) -> Div {
    div()
        .w(px(theme::TILE_MD * 3.0 + theme::TILE_GAP * 2.0))
        .h(px(theme::TILE_MD * 2.0 + theme::TILE_GAP))
        .flex()
        .flex_col()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.block))
        .p(px(18.0))
        .cursor_pointer()
}

/// Column tile used for the recent-worlds list.
pub fn list_tile(t: &ThemeTokens) -> Div {
    div()
        .w(px(theme::TILE_MD * 3.0 + theme::TILE_GAP * 2.0))
        .h(px(theme::TILE_MD * 2.0 + theme::TILE_GAP))
        .flex()
        .flex_col()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.surface))
        .border_1()
        .border_color(rgb(t.line))
        .p(px(18.0))
}

/// Persistent bottom action bar.
pub fn action_bar(t: &ThemeTokens) -> Div {
    div()
        .h(px(theme::ACTION_BAR_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(16.0))
        .px(px(24.0))
        .bg(rgb(t.surface))
        .border_t_1()
        .border_color(rgb(t.line))
}

/// Filled primary action.
pub fn primary_button(label: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    div()
        .h(px(58.0))
        .px(px(26.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.accent))
        .cursor_pointer()
        .child(
            div()
                .text_size(px(20.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.accent_text))
                .child(label),
        )
}

/// Quiet bordered button.
pub fn quiet_button(label: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    div()
        .h(px(40.0))
        .px(px(16.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .border_1()
        .border_color(rgb(t.line))
        .cursor_pointer()
        .hover(move |s| s.border_color(rgb(t.text)))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(t.text))
                .child(label),
        )
}

/// Bordered button that reports a current selection.
pub fn choice_button(label: impl Into<SharedString>, selected: bool, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    let (bg, fg, border) = if selected {
        (t.block, t.block_text, t.block)
    } else {
        (t.surface, t.text, t.line)
    };

    div()
        .h(px(40.0))
        .px(px(16.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(bg))
        .border_1()
        .border_color(rgb(border))
        .cursor_pointer()
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(fg))
                .child(label),
        )
}

/// Square icon button.
pub fn icon_button(icon: IconName, t: &ThemeTokens) -> Div {
    div()
        .w(px(38.0))
        .h(px(38.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .border_1()
        .border_color(rgb(t.line))
        .cursor_pointer()
        .hover(move |s| s.border_color(rgb(t.text)))
        .child(
            Icon::new(icon)
                .w(px(17.0))
                .h(px(17.0))
                .text_color(rgb(t.muted)),
        )
}

/// Square avatar placeholder.
pub fn avatar(letter: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let letter: SharedString = letter.into();
    div()
        .w(px(42.0))
        .h(px(42.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.block))
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(t.block_text))
                .child(letter),
        )
}

/// One row of a settings list.
pub fn field_row(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
    t: &ThemeTokens,
) -> Div {
    let label: SharedString = label.into();
    let value: SharedString = value.into();
    div()
        .h(px(48.0))
        .flex()
        .items_center()
        .justify_between()
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .child(
            div()
                .text_size(px(13.0))
                .text_color(rgb(t.text))
                .child(label),
        )
        .child(
            div()
                .text_size(px(13.0))
                .text_color(rgb(t.muted))
                .child(value),
        )
}
