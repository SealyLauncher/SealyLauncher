//! Downloads: version list with a source filter and a progress action bar.

use crate::app::SealyShell;
use crate::components::{action_bar, head_band, mono, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::ShellState;
use crate::theme::{self, ThemeTokens};

struct Version {
    id: &'static str,
    kind: &'static str,
    released: &'static str,
    size: &'static str,
}

const VERSIONS: [Version; 6] = [
    Version {
        id: "1.21.11",
        kind: "正式版",
        released: "11-08",
        size: "632 MB",
    },
    Version {
        id: "1.21.10",
        kind: "正式版",
        released: "10-24",
        size: "631 MB",
    },
    Version {
        id: "1.21.9",
        kind: "正式版",
        released: "10-10",
        size: "628 MB",
    },
    Version {
        id: "1.21.4",
        kind: "正式版",
        released: "2024-12-03",
        size: "596 MB",
    },
    Version {
        id: "1.20.1",
        kind: "正式版",
        released: "2023-06-12",
        size: "514 MB",
    },
    Version {
        id: "24w14a",
        kind: "快照",
        released: "2024-04-04",
        size: "588 MB",
    },
];

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let rows = VERSIONS
        .iter()
        .enumerate()
        .map(|(index, version)| row(index, version, lang, &t, cx))
        .collect::<Vec<_>>();

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band("03", tr(lang, "下载", "Downloads"), &t))
        .child(
            div()
                .flex_1()
                .p(px(theme::PAGE_PADDING))
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(44.0))
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .h(px(36.0))
                                .w(px(260.0))
                                .px(px(12.0))
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .rounded(px(theme::RADIUS_TILE))
                                .border_1()
                                .border_color(rgb(t.line))
                                .child(
                                    Icon::new(IconName::Search)
                                        .w(px(14.0))
                                        .h(px(14.0))
                                        .text_color(rgb(t.muted)),
                                )
                                .child(
                                    div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
                                        lang,
                                        "搜索版本",
                                        "Search versions",
                                    )),
                                ),
                        )
                        .child(div().flex_1())
                        .child(mono(tr(lang, "全部", "All"), &t)),
                )
                .child(div().w_full().h(px(1.0)).bg(rgb(t.line)))
                .children(rows),
        )
        .child(
            action_bar(&t)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(
                            div()
                                .w(px(160.0))
                                .h(px(4.0))
                                .rounded(px(2.0))
                                .bg(rgb(t.line_soft))
                                .child(
                                    div()
                                        .w(relative(0.38))
                                        .h_full()
                                        .rounded(px(2.0))
                                        .bg(rgb(t.text)),
                                ),
                        )
                        .child(mono(tr(lang, "38%", "38%"), &t)),
                )
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "取消", "Cancel"), &t)
                        .id("download-cancel")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.notify(tr(
                                shell.state.language(),
                                "下载将在 Phase 2 接入",
                                "Downloads arrive in Phase 2",
                            ));
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

fn row(
    index: usize,
    version: &Version,
    lang: sealy_core::config::Language,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    div()
        .h(px(60.0))
        .flex()
        .items_center()
        .gap(px(16.0))
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .child(
            div()
                .w(px(120.0))
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(t.text))
                .child(version.id),
        )
        .child(
            div()
                .w(px(72.0))
                .text_size(px(11.0))
                .text_color(rgb(t.muted))
                .child(tr(
                    lang,
                    version.kind,
                    if version.kind == "快照" {
                        "Snapshot"
                    } else {
                        "Release"
                    },
                )),
        )
        .child(div().flex_1())
        .child(mono(version.released, t))
        .child(mono(version.size, t))
        .child(
            quiet_button(tr(lang, "安装", "Install"), t)
                .id(("install", index))
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.state.notify(tr(
                        shell.state.language(),
                        "下载将在 Phase 2 接入",
                        "Downloads arrive in Phase 2",
                    ));
                    cx.notify();
                })),
        )
        .into_any_element()
}
