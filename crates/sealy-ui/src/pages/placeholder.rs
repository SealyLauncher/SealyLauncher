//! Placeholder pages: one quiet state until their phase lands.

use crate::app::SealyShell;
use crate::components::{action_bar, head_band, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::ShellState;

/// Page body shared by every unfinished route.
pub(crate) fn placeholder(
    index: &str,
    title: &'static str,
    icon: IconName,
    line: &'static str,
    action: &'static str,
    state: &ShellState,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let t = state.theme();

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band(index, title, &t))
        .child(
            div().flex_1().flex().items_center().justify_center().child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(18.0))
                    .child(
                        Icon::new(icon)
                            .w(px(40.0))
                            .h(px(40.0))
                            .text_color(rgb(t.faint)),
                    )
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(t.muted))
                            .child(line),
                    )
                    .child(quiet_button(action, &t).id("placeholder-action").on_click(
                        cx.listener(|shell, _, _, cx| {
                            shell.state.notify(tr(
                                shell.state.language(),
                                "该功能将在后续阶段接入",
                                "This arrives in a later phase",
                            ));
                            cx.notify();
                        }),
                    )),
            ),
        )
        .child(
            action_bar(&t).child(div().flex_1()).child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(t.faint))
                    .child("Phase 2"),
            ),
        )
        .into_any_element()
}

/// Renders one placeholder page.
pub(crate) fn render_instances(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    placeholder(
        "02",
        tr(state.language(), "实例", "Instances"),
        IconName::Layers,
        tr(state.language(), "还没有创建实例", "No instances yet"),
        tr(state.language(), "创建实例", "Create instance"),
        state,
        cx,
    )
}

/// Renders the downloads placeholder.
pub(crate) fn render_downloads(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    placeholder(
        "03",
        tr(state.language(), "下载", "Downloads"),
        IconName::Download,
        tr(
            state.language(),
            "版本列表将在 Phase 2 接入",
            "Version list arrives in Phase 2",
        ),
        tr(state.language(), "浏览版本", "Browse versions"),
        state,
        cx,
    )
}

/// Renders the accounts placeholder.
pub(crate) fn render_accounts(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    placeholder(
        "04",
        tr(state.language(), "账号", "Accounts"),
        IconName::CircleUser,
        tr(state.language(), "还没有添加账号", "No accounts yet"),
        tr(state.language(), "添加账号", "Add account"),
        state,
        cx,
    )
}
