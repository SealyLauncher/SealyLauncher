//! Settings page: flat rows, no section headings, real portable path display.

use sealy_core::config::Language;

use crate::app::SealyShell;
use crate::components;
use crate::i18n;
use crate::kit::*;
use crate::state::ShellState;
use crate::theme;

fn row(label: impl Into<SharedString>, control: impl IntoElement) -> AnyElement {
    let label: SharedString = label.into();
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .py_4()
        .child(
            div()
                .text_size(px(14.0))
                .text_color(rgb(theme::TEXT))
                .child(label),
        )
        .child(control.into_any_element())
        .into_any_element()
}

/// Renders settings.
pub fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();
    let zh_active = lang == Language::ZhCn;
    let en_active = lang == Language::EnUs;

    let language_control = div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            components::segment("中文", zh_active)
                .id("language-zh")
                .cursor_pointer()
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.state.set_language(Language::ZhCn);
                    cx.notify();
                })),
        )
        .child(
            components::segment("English", en_active)
                .id("language-en")
                .cursor_pointer()
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.state.set_language(Language::EnUs);
                    cx.notify();
                })),
        );

    let mut rows: Vec<AnyElement> = Vec::new();
    rows.push(row(i18n::settings_language(lang), language_control));
    rows.push(components::separator().into_any_element());
    rows.push(row(
        i18n::settings_theme(lang),
        components::pill(i18n::settings_dark(lang)),
    ));
    rows.push(components::separator().into_any_element());
    rows.push(row(
        i18n::settings_motion(lang),
        components::pill(i18n::settings_system(lang)),
    ));
    rows.push(components::separator().into_any_element());
    rows.push(row(
        i18n::settings_source(lang),
        components::pill(i18n::settings_auto(lang)),
    ));
    rows.push(components::separator().into_any_element());
    rows.push(row(
        i18n::settings_oem(lang),
        components::pill(i18n::settings_oem_off(lang)),
    ));
    rows.push(components::separator().into_any_element());
    rows.push(row(
        i18n::settings_data_dir(lang),
        div()
            .max_w(px(420.0))
            .text_size(px(11.0))
            .text_color(rgb(theme::MUTED))
            .child(state.paths.data.display().to_string()),
    ));
    rows.push(components::separator().into_any_element());
    rows.push(row(
        i18n::settings_version(lang),
        components::pill(format!("SealyLauncher {}", sealy_core::version())),
    ));

    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p(px(theme::PAGE_PADDING))
        .child(
            div()
                .w(px(theme::CONTENT_MAX_WIDTH))
                .child(components::glass_panel().p(px(24.0)).children(rows)),
        )
        .into_any_element()
}
