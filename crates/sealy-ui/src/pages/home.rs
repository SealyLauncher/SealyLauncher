//! Home page: one immersive display title and the primary next action.

use crate::app::SealyShell;
use crate::components;
use crate::i18n;
use crate::kit::*;
use crate::state::{Route, ShellState};
use crate::theme;

/// Renders the home page.
pub fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();
    let account_label = i18n::sign_in(lang);

    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p(px(theme::PAGE_PADDING))
        .child(
            div()
                .w(px(720.0))
                .flex()
                .flex_col()
                .items_start()
                .gap_6()
                .child(
                    div()
                        .text_size(px(56.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(theme::TEXT))
                        .child(i18n::home_title(lang)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(components::pill(i18n::home_status(lang)))
                        .child(components::pill("Minecraft")),
                )
                .child(
                    components::primary_button(i18n::install_version(lang))
                        .id("home-install")
                        .cursor_pointer()
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.route = Route::Downloads;
                            cx.notify();
                        })),
                )
                .child(
                    components::glass_panel()
                        .id("home-account")
                        .role(Role::Button)
                        .aria_label(account_label)
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_4()
                        .py_3()
                        .cursor_pointer()
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.route = Route::Accounts;
                            cx.notify();
                        }))
                        .child(
                            div()
                                .w(px(36.0))
                                .h(px(36.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .bg(rgb(theme::SURFACE_RAISED))
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(theme::ACCENT))
                                        .child("S"),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(theme::TEXT))
                                .child(account_label),
                        ),
                ),
        )
        .into_any_element()
}
