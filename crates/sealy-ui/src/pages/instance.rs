//! Detail view of one instance: overview, launch actions and addon lists.

use crate::app::SealyShell;
use crate::components::{
    action_bar, field_row, head_band_back_to, mono, primary_button, quiet_button,
};
use crate::i18n::tr;
use crate::kit::*;
use crate::launch_flow;
use crate::state::{Route, ShellState};
use crate::theme::{self, ThemeTokens};

/// Renders the detail page of the selected instance.
pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let Some(instance) = state.instances.current() else {
        return empty(state, &t, lang, cx);
    };

    let t = state.theme();
    let directory = sealy_core::instances::directory(&state.paths.game, &instance.name);
    let java_line = state
        .java
        .first()
        .map(|java| format!("Java {} · {}", java.version, java.path.display()))
        .unwrap_or_else(|| tr(lang, "自动选择", "Picked automatically").to_owned());
    let memory = instance
        .launch
        .max_mb
        .unwrap_or(state.settings.java.memory.max_mb);
    let width = instance.launch.width.unwrap_or(state.settings.launch.width);
    let height = instance
        .launch
        .height
        .unwrap_or(state.settings.launch.height);
    let jvm = if instance.launch.jvm_args.trim().is_empty() {
        tr(lang, "默认", "Default").to_owned()
    } else {
        instance.launch.jvm_args.clone()
    };
    let played = instance
        .last_played
        .map(played_label)
        .unwrap_or_else(|| tr(lang, "从未启动", "Never launched").to_owned());

    let body = div()
        .flex_1()
        .min_h(px(0.0))
        .id("instance-detail-body")
        .overflow_y_scroll()
        .p(px(theme::PAGE_PADDING))
        .flex()
        .flex_col()
        .gap(px(18.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(16.0))
                .child(
                    div()
                        .w(px(64.0))
                        .h(px(64.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(theme::RADIUS_TILE))
                        .border_1()
                        .border_color(rgb(t.line))
                        .child(
                            Icon::new(IconName::Box)
                                .w(px(28.0))
                                .h(px(28.0))
                                .text_color(rgb(t.text)),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(24.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(t.text))
                                .child(instance.name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(t.muted))
                                .child(instance.version_id.clone()),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .border_t_1()
                .border_color(rgb(t.line_soft))
                .child(field_row(
                    tr(lang, "版本", "Version"),
                    instance.version_id.clone(),
                    &t,
                ))
                .child(field_row(
                    tr(lang, "实例目录", "Instance folder"),
                    directory.display().to_string(),
                    &t,
                ))
                .child(field_row(tr(lang, "最后启动", "Last launched"), played, &t))
                .child(field_row(tr(lang, "Java", "Java"), java_line, &t))
                .child(field_row(
                    tr(lang, "内存 / 窗口", "Memory / window"),
                    format!("{memory} MB · {width} × {height}"),
                    &t,
                ))
                .child(field_row(tr(lang, "JVM 参数", "JVM arguments"), jvm, &t)),
        )
        .into_any_element();

    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .child(head_band_back_to(
            instance.name.clone(),
            Route::Instances,
            &t,
            cx,
        ))
        .child(body)
        .child(
            action_bar(&t)
                .child(mono(instance.version_id.clone(), &t))
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "日志", "Logs"), &t)
                        .id("detail-logs")
                        .role(Role::Button)
                        .aria_label("detail-logs")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.open_logs();
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "启动设置", "Launch settings"), &t)
                        .id("detail-settings")
                        .role(Role::Button)
                        .aria_label("detail-settings")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.begin_instance_settings();
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "打开目录", "Open folder"), &t)
                        .id("detail-folder")
                        .role(Role::Button)
                        .aria_label("detail-folder")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.open_instance_folder();
                            cx.notify();
                        })),
                )
                .child(
                    primary_button(tr(lang, "启动游戏", "Launch"), &t)
                        .id("detail-launch")
                        .role(Role::Button)
                        .aria_label("detail-launch")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            launch_flow::start_selected(&mut shell.state, cx);
                        })),
                ),
        )
        .children(
            state
                .instance_settings_open
                .then(|| super::instances::launch_settings_panel(state, &t, lang, cx)),
        )
        .children(
            state
                .logs_open
                .then(|| super::instances::log_panel(state, &t, lang, cx)),
        )
        .into_any_element()
}

/// Empty state when there is no instance to show.
fn empty(
    _state: &ShellState,
    t: &ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band_back_to(
            tr(lang, "实例详情", "Instance"),
            Route::Instances,
            t,
            cx,
        ))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(10.0))
                .child(
                    Icon::new(IconName::Box)
                        .w(px(28.0))
                        .h(px(28.0))
                        .text_color(rgb(t.faint)),
                )
                .child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
                    lang,
                    "没有可显示的实例",
                    "No instance to show",
                ))),
        )
        .into_any_element()
}

/// Human label for a last-played timestamp.
fn played_label(stamp: u64) -> String {
    let now = sealy_core::auth::now_epoch();
    let delta = now.saturating_sub(stamp);
    if delta < 60 {
        "刚刚".to_owned()
    } else if delta < 3_600 {
        format!("{} 分钟前", delta / 60)
    } else if delta < 86_400 {
        format!("{} 小时前", delta / 3_600)
    } else {
        format!("{} 天前", delta / 86_400)
    }
}
