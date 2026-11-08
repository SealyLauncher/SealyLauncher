//! Settings: theme presets, language and facts about the portable root.

use sealy_core::config::{Language, ThemeName};

use crate::app::SealyShell;
use crate::components::{action_bar, choice_button, field_row, head_band, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::ShellState;
use crate::theme::{self, ThemeTokens};

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let swatches = ThemeName::ALL
        .into_iter()
        .map(|name| swatch(name, state.theme_name(), lang, &t, cx))
        .collect::<Vec<_>>();

    let data_dir = state.paths.root.display().to_string();

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band("05", "设置", &t))
        .child(
            div()
                .flex_1()
                .p(px(theme::PAGE_PADDING))
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(t.text))
                                .child(tr(lang, "主题", "Theme")),
                        )
                        .child(div().flex().gap(px(10.0)).children(swatches)),
                )
                .child(div().h(px(28.0)))
                .child(
                    div()
                        .w(px(720.0))
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .h(px(56.0))
                                .flex()
                                .items_center()
                                .justify_between()
                                .border_b_1()
                                .border_color(rgb(t.line_soft))
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(rgb(t.text))
                                        .child(tr(lang, "语言", "Language")),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.0))
                                        .child(
                                            choice_button("简体中文", lang == Language::ZhCn, &t)
                                                .id("lang-zh")
                                                .on_click(cx.listener(|shell, _, _, cx| {
                                                    shell.state.set_language(Language::ZhCn);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            choice_button("English", lang == Language::EnUs, &t)
                                                .id("lang-en")
                                                .on_click(cx.listener(|shell, _, _, cx| {
                                                    shell.state.set_language(Language::EnUs);
                                                    cx.notify();
                                                })),
                                        ),
                                ),
                        )
                        .child(field_row(
                            tr(lang, "数据目录", "Data directory"),
                            data_dir,
                            &t,
                        ))
                        .child(field_row(
                            tr(lang, "版本", "Version"),
                            format!("{} · portable", sealy_core::version()),
                            &t,
                        ))
                        .child(
                            div()
                                .h(px(56.0))
                                .flex()
                                .items_center()
                                .justify_between()
                                .border_b_1()
                                .border_color(rgb(t.line_soft))
                                .child(div().text_size(px(13.0)).text_color(rgb(t.text)).child(tr(
                                    lang,
                                    "设置向导",
                                    "Setup wizard",
                                )))
                                .child(
                                    quiet_button(tr(lang, "重新运行", "Run again"), &t)
                                        .id("restart-oobe")
                                        .on_click(cx.listener(|shell, _, _, cx| {
                                            shell.state.start_oobe();
                                            cx.notify();
                                        })),
                                ),
                        ),
                ),
        )
        .child(
            action_bar(&t).child(div().flex_1()).child(
                quiet_button(tr(lang, "打开数据目录", "Open data folder"), &t)
                    .id("open-data")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.notify("打开目录将在 Phase 2 接入");
                        cx.notify();
                    })),
            ),
        )
        .into_any_element()
}

fn swatch(
    name: ThemeName,
    active: ThemeName,
    lang: Language,
    current: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let preset = ThemeTokens::preset(name);
    let selected = name == active;
    let index = ThemeName::ALL
        .iter()
        .position(|candidate| *candidate == name)
        .unwrap_or(0);

    let label = match name {
        ThemeName::Paper => tr(lang, "纸张", "Paper"),
        ThemeName::Ink => tr(lang, "墨色", "Ink"),
        ThemeName::Clay => tr(lang, "陶土", "Clay"),
        ThemeName::Celadon => tr(lang, "青瓷", "Celadon"),
    };

    div()
        .id(("theme", index))
        .role(Role::Button)
        .aria_label(label)
        .w(px(96.0))
        .h(px(64.0))
        .p(px(10.0))
        .flex()
        .flex_col()
        .justify_between()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(preset.bg))
        .border_1()
        .border_color(if selected {
            rgb(current.text)
        } else {
            rgb(current.line)
        })
        .cursor_pointer()
        .hover(move |s| s.border_color(rgb(current.muted)))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(
                    div()
                        .w(px(28.0))
                        .h(px(12.0))
                        .rounded(px(1.0))
                        .bg(rgb(preset.block)),
                )
                .child(
                    div()
                        .w(px(10.0))
                        .h(px(10.0))
                        .rounded(px(1.0))
                        .bg(rgb(preset.accent)),
                ),
        )
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(preset.text))
                .child(label),
        )
        .on_click(cx.listener(move |shell, _, _, cx| {
            shell.state.set_theme(name);
            cx.notify();
        }))
        .into_any_element()
}
