//! Instances page: empty state until Phase 2 owns instance storage.

use crate::app::SealyShell;
use crate::components;
use crate::i18n;
use crate::kit::*;
use crate::state::{Route, ShellState};
use crate::theme;

/// Renders the instance list.
pub fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();

    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p(px(theme::PAGE_PADDING))
        .child(
            components::glass_panel()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .px(px(56.0))
                .py(px(48.0))
                .child(components::icon_tile(IconName::FolderClosed))
                .child(
                    div()
                        .text_size(px(24.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::TEXT))
                        .child(i18n::no_instances(lang)),
                )
                .child(
                    components::primary_button(i18n::install_version(lang))
                        .id("instances-install")
                        .cursor_pointer()
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.route = Route::Downloads;
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}
