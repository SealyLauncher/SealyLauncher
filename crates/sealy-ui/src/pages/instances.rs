//! Instances: a scannable list with a persistent launch action.

use crate::app::SealyShell;
use crate::components::{action_bar, head_band, mono, primary_button, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::ShellState;
use crate::theme::{self, ThemeTokens};

struct Instance {
    name: &'static str,
    version: &'static str,
    loader: &'static str,
    played: &'static str,
    size: &'static str,
}

const INSTANCES: [Instance; 4] = [
    Instance {
        name: "Fabric 1.21.11",
        version: "1.21.11",
        loader: "Fabric 0.16.5",
        played: "2H",
        size: "812 MB",
    },
    Instance {
        name: "Aurora",
        version: "1.20.1",
        loader: "128 mods",
        played: "3D",
        size: "3.4 GB",
    },
    Instance {
        name: "Vanilla 1.21.4",
        version: "1.21.4",
        loader: "Vanilla",
        played: "1W",
        size: "498 MB",
    },
    Instance {
        name: "Skyblock",
        version: "1.20.1",
        loader: "Fabric 0.15.9",
        played: "2W",
        size: "1.2 GB",
    },
];

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let rows = INSTANCES
        .iter()
        .enumerate()
        .map(|(index, instance)| row(instance, index == 0, &t))
        .collect::<Vec<_>>();

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band("02", tr(lang, "实例", "Instances"), &t))
        .child(
            div()
                .flex_1()
                .p(px(theme::PAGE_PADDING))
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(40.0))
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
                            lang,
                            "全部 · 按最近使用排序",
                            "All · most recent first",
                        )))
                        .child(mono(format!("{:02}", INSTANCES.len()), &t)),
                )
                .child(div().w_full().h(px(1.0)).bg(rgb(t.line)))
                .children(rows),
        )
        .child(
            action_bar(&t)
                .child(mono(tr(lang, "4 个实例", "4 instances"), &t))
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "编辑", "Edit"), &t)
                        .id("instance-edit")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.notify(tr(
                                shell.state.language(),
                                "编辑实例将在 Phase 2 接入",
                                "Editing arrives in Phase 2",
                            ));
                            cx.notify();
                        })),
                )
                .child(
                    primary_button(tr(lang, "启动", "Launch"), &t)
                        .id("instance-launch")
                        .role(Role::Button)
                        .aria_label(tr(lang, "启动", "Launch"))
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.notify(tr(
                                shell.state.language(),
                                "启动流程将在 Phase 2 接入",
                                "Launching arrives in Phase 2",
                            ));
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

fn row(instance: &Instance, selected: bool, t: &ThemeTokens) -> AnyElement {
    div()
        .h(px(72.0))
        .flex()
        .items_center()
        .gap(px(14.0))
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .when(selected, |this| this.bg(rgb(t.surface)))
        .child(
            div()
                .w(px(40.0))
                .h(px(40.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.line))
                .child(
                    Icon::new(IconName::Box)
                        .w(px(18.0))
                        .h(px(18.0))
                        .text_color(rgb(t.text)),
                ),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(3.0))
                .child(
                    div()
                        .text_size(px(15.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(instance.name),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(rgb(t.muted))
                        .child(format!("{} · {}", instance.version, instance.loader)),
                ),
        )
        .child(mono(instance.size, t))
        .child(div().w(px(28.0)))
        .child(mono(instance.played, t))
        .child(
            Icon::new(IconName::ChevronRight)
                .w(px(14.0))
                .h(px(14.0))
                .text_color(rgb(t.faint)),
        )
        .into_any_element()
}
