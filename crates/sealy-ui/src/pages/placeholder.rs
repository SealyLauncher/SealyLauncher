//! Placeholder pages: one quiet state until their phase lands.

use crate::app::SealyShell;
use crate::components::{action_bar, head_band, quiet_button};
use crate::kit::*;
use crate::state::ShellState;
use crate::theme;

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
                            shell.state.notify("该功能将在后续阶段接入");
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
        "实例",
        IconName::Layers,
        "还没有创建实例",
        "创建实例",
        state,
        cx,
    )
}

/// Renders the downloads placeholder.
pub(crate) fn render_downloads(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    placeholder(
        "03",
        "下载",
        IconName::Download,
        "版本列表将在 Phase 2 接入",
        "浏览版本",
        state,
        cx,
    )
}

/// Renders the accounts placeholder.
pub(crate) fn render_accounts(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    placeholder(
        "04",
        "账号",
        IconName::CircleUser,
        "还没有添加账号",
        "添加账号",
        state,
        cx,
    )
}

#[allow(dead_code)]
fn _unused(_: theme::ThemeTokens) {}
