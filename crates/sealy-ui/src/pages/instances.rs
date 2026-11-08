//! Instances: the launchable profiles and their own game directories.

use sealy_core::instances::Instance;

use crate::app::SealyShell;
use crate::components::{action_bar, head_band_back, mono, primary_button, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::launch_flow;
use crate::state::ShellState;
use crate::theme::{self, ThemeTokens};

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let rows = state
        .instances
        .instances
        .iter()
        .map(|instance| row(instance, state, &t, cx))
        .collect::<Vec<_>>();

    let body = if state.new_instance_open {
        picker(state, &t, lang, cx)
    } else if rows.is_empty() {
        empty(&t, lang)
    } else {
        div()
            .id("instance-list")
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .children(rows)
            .into_any_element()
    };

    let has_selection = state.instances.current().is_some();
    let count_label = format!(
        "{} {}",
        state.instances.instances.len(),
        tr(lang, "个实例", "instances")
    );

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band_back("02", tr(lang, "实例", "Instances"), &t, cx))
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
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
                            "每个实例拥有独立的存档与设置",
                            "Each instance keeps its own saves and options",
                        )))
                        .child(mono(format!("{:02}", state.instances.instances.len()), &t)),
                )
                .child(div().w_full().h(px(1.0)).bg(rgb(t.line)))
                .child(body),
        )
        .child(
            action_bar(&t)
                .child(mono(count_label, &t))
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "删除", "Delete"), &t)
                        .id("instance-delete")
                        .role(Role::Button)
                        .aria_label("delete-instance")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            if let Some(instance) = shell.state.instances.current().cloned() {
                                shell.state.remove_instance(&instance.name);
                            }
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "新建实例", "New instance"), &t)
                        .id("instance-new")
                        .role(Role::Button)
                        .aria_label("new-instance")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            if shell.state.installed.is_empty() {
                                let lang = shell.state.language();
                                shell.state.notify(tr(
                                    lang,
                                    "请先在下载页安装一个版本",
                                    "Install a version from Downloads first",
                                ));
                            } else {
                                shell.state.new_instance_open = true;
                            }
                            cx.notify();
                        })),
                )
                .child(
                    primary_button(tr(lang, "启动", "Launch"), &t)
                        .id("instance-launch")
                        .role(Role::Button)
                        .aria_label(tr(lang, "启动", "Launch"))
                        .when(!has_selection, |this| this.opacity(0.4))
                        .on_click(cx.listener(|shell, _, _, cx| {
                            launch_flow::start_selected(&mut shell.state, cx);
                        })),
                ),
        )
        .into_any_element()
}

/// Version picker shown while creating an instance.
fn picker(
    state: &ShellState,
    t: &ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let mut choices = div().flex().flex_wrap().gap(px(10.0));

    if state.installed.is_empty() {
        choices = choices.child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
            lang,
            "还没有已安装的版本，请先去下载页安装",
            "No installed version — install one from Downloads",
        )));
    } else {
        for version in state.installed.iter().take(8) {
            let id = version.id.clone();
            choices = choices.child(
                quiet_button(version.id.clone(), t)
                    .id(("pick", version.id.len()))
                    .role(Role::Button)
                    .aria_label(format!("pick {}", version.id))
                    .on_click(cx.listener(move |shell, _, _, cx| {
                        shell.state.create_instance(&id);
                        shell.state.new_instance_open = false;
                        cx.notify();
                    })),
            );
        }
    }

    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(16.0))
        .pt(px(22.0))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "选择版本", "Choose a version")),
        )
        .child(choices)
        .child(
            div().flex().child(
                quiet_button(tr(lang, "取消", "Cancel"), t)
                    .id("pick-cancel")
                    .role(Role::Button)
                    .aria_label("pick-cancel")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.new_instance_open = false;
                        cx.notify();
                    })),
            ),
        )
        .into_any_element()
}

fn empty(t: &ThemeTokens, lang: sealy_core::config::Language) -> AnyElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .child(
            Icon::new(IconName::Layers)
                .w(px(28.0))
                .h(px(28.0))
                .text_color(rgb(t.faint)),
        )
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "还没有实例", "No instances yet")),
        )
        .child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
            lang,
            "安装版本后点「新建实例」即可创建",
            "Install a version, then press “New instance”",
        )))
        .into_any_element()
}

fn row(
    instance: &Instance,
    state: &ShellState,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let selected = state.instances.selected.as_deref() == Some(instance.name.as_str());
    let name = instance.name.clone();
    let shown_name = instance.name.clone();
    let meta = instance.version_id.clone();
    let played = instance
        .last_played
        .map(age_label)
        .unwrap_or_else(|| "—".to_owned());

    div()
        .id(("instance", instance.name.len()))
        .role(Role::Button)
        .aria_label(instance.name.clone())
        .h(px(72.0))
        .flex()
        .items_center()
        .gap(px(14.0))
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .cursor_pointer()
        .when(selected, |this| this.bg(rgb(t.surface)))
        .hover(move |this| this.bg(rgb(t.surface)))
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
                        .child(shown_name),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(rgb(t.muted))
                        .child(meta),
                ),
        )
        .child(mono(played, t))
        .child(
            Icon::new(IconName::ChevronRight)
                .w(px(14.0))
                .h(px(14.0))
                .text_color(rgb(t.faint)),
        )
        .on_click(cx.listener(move |shell, _, _, cx| {
            shell.state.select_instance(&name);
            cx.notify();
        }))
        .into_any_element()
}

/// `2H · 3D · 1W` style label for the last launch.
fn age_label(stamp: u64) -> String {
    let now = launch_flow::now_epoch();
    let delta = now.saturating_sub(stamp);
    if delta < 60 {
        "1M".to_owned()
    } else if delta < 3_600 {
        format!("{}M", delta / 60)
    } else if delta < 86_400 {
        format!("{}H", delta / 3_600)
    } else {
        format!("{}D", delta / 86_400)
    }
}
