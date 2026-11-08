//! Accounts page: three provider choices, no explanatory headings.

use crate::app::SealyShell;
use crate::components;
use crate::i18n;
use crate::kit::*;
use crate::state::ShellState;
use crate::theme;

/// Renders account method cards.
pub fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();
    let methods = [
        (IconName::CircleUser, i18n::microsoft(lang)),
        (IconName::Globe, i18n::authlib(lang)),
        (IconName::User, i18n::offline(lang)),
    ];
    let notice = i18n::toast_login_phase3(lang);

    let cards = methods
        .iter()
        .enumerate()
        .map(|(index, (icon, label))| {
            components::glass_panel()
                .id(("account-method", index))
                .role(Role::Button)
                .aria_label(*label)
                .flex()
                .items_center()
                .gap_4()
                .p(px(18.0))
                .cursor_pointer()
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.state.notify(notice);
                    cx.notify();
                }))
                .child(components::icon_tile(*icon))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(15.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(theme::TEXT))
                        .child(*label),
                )
                .child(
                    Icon::new(IconName::ChevronRight)
                        .w(px(16.0))
                        .h(px(16.0))
                        .text_color(rgb(theme::FAINT)),
                )
        })
        .collect::<Vec<_>>();

    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p(px(theme::PAGE_PADDING))
        .child(
            div()
                .w(px(theme::CONTENT_MAX_WIDTH))
                .flex()
                .flex_col()
                .gap_3()
                .children(cards),
        )
        .into_any_element()
}
