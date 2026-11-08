//! Downloads page: Phase 1 visual skeleton with mock version rows.

use crate::app::SealyShell;
use crate::components;
use crate::i18n;
use crate::kit::*;
use crate::state::ShellState;
use crate::theme;

#[derive(Clone, Copy)]
enum VersionKind {
    Release,
    Snapshot,
}

const VERSIONS: [(&str, VersionKind); 8] = [
    ("1.21.4", VersionKind::Release),
    ("1.21.3", VersionKind::Release),
    ("1.20.6", VersionKind::Release),
    ("1.20.1", VersionKind::Release),
    ("1.19.4", VersionKind::Release),
    ("1.18.2", VersionKind::Release),
    ("1.16.5", VersionKind::Release),
    ("24w14a", VersionKind::Snapshot),
];

/// Renders the version browser.
pub fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();
    let rows = VERSIONS
        .iter()
        .enumerate()
        .map(|(index, (version, kind))| {
            let kind_label = match kind {
                VersionKind::Release => i18n::release_label(lang),
                VersionKind::Snapshot => i18n::snapshot_label(lang),
            };
            let notice = i18n::toast_install_phase2(lang);

            div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            Icon::new(IconName::Download)
                                .w(px(16.0))
                                .h(px(16.0))
                                .text_color(rgb(theme::MUTED)),
                        )
                        .child(
                            div()
                                .text_size(px(16.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(theme::TEXT))
                                .child(*version),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(components::pill(kind_label))
                        .child(
                            components::secondary_button(i18n::install(lang))
                                .id(("download-install", index))
                                .cursor_pointer()
                                .on_click(cx.listener(move |shell, _, _, cx| {
                                    shell.state.notify(notice);
                                    cx.notify();
                                })),
                        ),
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
                .gap_4()
                .child(
                    div()
                        .h(px(42.0))
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .rounded(px(theme::RADIUS_MD))
                        .bg(rgba(0xFFFFFF08))
                        .border_1()
                        .border_color(rgba(0xFFFFFF12))
                        .child(
                            Icon::new(IconName::Search)
                                .w(px(15.0))
                                .h(px(15.0))
                                .text_color(rgb(theme::MUTED)),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(theme::FAINT))
                                .child(i18n::search_versions(lang)),
                        ),
                )
                .child(
                    components::glass_panel()
                        .p(px(6.0))
                        .flex()
                        .flex_col()
                        .children(rows),
                ),
        )
        .into_any_element()
}
