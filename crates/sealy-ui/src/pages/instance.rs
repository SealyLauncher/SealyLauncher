//! Detail view of one instance: overview, launch actions and addon lists.

use crate::app::SealyShell;
use crate::components::{
    action_bar, choice_button, field_row, head_band_back_to, mono, primary_button, quiet_button,
};
use crate::i18n::tr;
use crate::kit::*;
use crate::launch_flow;
use crate::state::{Route, ShellState};
use crate::theme::{self, ThemeTokens};
use gpui_kit::ExternalPaths;

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

    let icon_choices = sealy_core::instances::ICON_KEYS
        .into_iter()
        .enumerate()
        .map(|(index, key)| {
            let selected = instance.icon.as_deref() == Some(key);
            let label = match key {
                "layers" => tr(lang, "层叠", "Layers"),
                "package" => tr(lang, "包裹", "Package"),
                "server" => tr(lang, "服务器", "Server"),
                _ => tr(lang, "盒子", "Box"),
            };
            choice_button(label, selected, &t)
                .id(("instance-icon", index))
                .role(Role::Button)
                .aria_label(format!("instance-icon-{key}"))
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.state.set_instance_icon(Some(key));
                    cx.notify();
                }))
        })
        .collect::<Vec<_>>();

    let overview = div()
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
                .child(field_row(tr(lang, "JVM 参数", "JVM arguments"), jvm, &t))
                .child(
                    div()
                        .h(px(52.0))
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_b_1()
                        .border_color(rgb(t.line_soft))
                        .child(div().text_size(px(13.0)).text_color(rgb(t.text)).child(tr(
                            lang,
                            "实例图标",
                            "Instance icon",
                        )))
                        .child(div().flex().gap(px(8.0)).children(icon_choices)),
                ),
        )
        .into_any_element();

    let content: AnyElement = match state.instance_section {
        crate::state::InstanceSection::Overview => overview,
        crate::state::InstanceSection::Mods => {
            addon_section(state, sealy_core::addons::AddonKind::Mod, &t, lang, cx)
        }
        crate::state::InstanceSection::ResourcePacks => addon_section(
            state,
            sealy_core::addons::AddonKind::ResourcePack,
            &t,
            lang,
            cx,
        ),
        crate::state::InstanceSection::Shaders => {
            addon_section(state, sealy_core::addons::AddonKind::Shader, &t, lang, cx)
        }
    };

    let body = div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .child(instance_sidebar(state, &t, lang, cx))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .id("instance-detail-body")
                .overflow_y_scroll()
                .p(px(theme::PAGE_PADDING))
                .child(content),
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

/// Left navigation of the instance page.
fn instance_sidebar(
    state: &ShellState,
    t: &ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let current = state.instance_section;
    let items = crate::state::InstanceSection::ALL
        .into_iter()
        .enumerate()
        .map(|(index, section)| {
            let selected = section == current;
            div()
                .id(("instance-section", index))
                .role(Role::Button)
                .aria_label(format!("instance-section-{index}"))
                .h(px(40.0))
                .w_full()
                .px(px(16.0))
                .flex()
                .items_center()
                .gap(px(10.0))
                .cursor_pointer()
                .border_l_1()
                .border_color(rgb(if selected { t.accent } else { t.surface }))
                .bg(rgb(if selected { t.surface } else { t.bg }))
                .hover({
                    let hover = t.line_soft;
                    move |this| this.bg(rgb(hover))
                })
                .text_size(px(12.0))
                .text_color(rgb(if selected { t.accent } else { t.text }))
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.state.instance_section = section;
                    cx.notify();
                }))
                .child(section.label(lang))
        })
        .collect::<Vec<_>>();

    div()
        .w(px(176.0))
        .flex_shrink_0()
        .h_full()
        .py(px(12.0))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .border_r_1()
        .border_color(rgb(t.line))
        .children(items)
        .into_any_element()
}
/// One addon section: title, count, import action and the entry list.
fn addon_section(
    state: &ShellState,
    kind: sealy_core::addons::AddonKind,
    t: &ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let title = match kind {
        sealy_core::addons::AddonKind::Mod => tr(lang, "模组", "Mods"),
        sealy_core::addons::AddonKind::ResourcePack => tr(lang, "资源包", "Resource packs"),
        sealy_core::addons::AddonKind::Shader => tr(lang, "光影", "Shaders"),
    };

    let folder = state.current_instance_dir();
    let items = folder
        .as_ref()
        .map(|dir| sealy_core::addons::list(dir, kind))
        .unwrap_or_default();
    let rows = items
        .iter()
        .enumerate()
        .map(|(index, addon)| {
            let name = addon.name.clone();
            let toggle_name = name.clone();
            let remove_name = name.clone();
            let state_label = if addon.enabled {
                tr(lang, "禁用", "Disable")
            } else {
                tr(lang, "启用", "Enable")
            };
            let size = if addon.is_dir {
                tr(lang, "目录", "Folder").to_owned()
            } else {
                format!("{} KB", addon.bytes.div_ceil(1024).max(1))
            };
            div()
                .h(px(44.0))
                .flex()
                .items_center()
                .gap(px(10.0))
                .border_b_1()
                .border_color(rgb(t.line_soft))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(if addon.enabled { t.text } else { t.faint }))
                                .child(name),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(rgb(t.faint))
                                .child(size),
                        ),
                )
                .child(
                    quiet_button(state_label, t)
                        .id(("addon-toggle", (kind as usize) * 100 + index))
                        .role(Role::Button)
                        .aria_label(format!("toggle-addon-{index}"))
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            match shell.state.toggle_addon(kind, &toggle_name) {
                                Ok(enabled) => {
                                    let text = if enabled {
                                        tr(lang, "已启用", "Enabled").to_owned()
                                    } else {
                                        tr(lang, "已禁用", "Disabled").to_owned()
                                    };
                                    shell.state.notify(format!("{text} {toggle_name}"));
                                }
                                Err(err) => shell.state.notify(err),
                            }
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "删除", "Delete"), t)
                        .id(("addon-remove", (kind as usize) * 100 + index))
                        .role(Role::Button)
                        .aria_label(format!("remove-addon-{index}"))
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            match shell.state.remove_addon(kind, &remove_name) {
                                Ok(()) => shell.state.notify(format!("已删除 {}", remove_name)),
                                Err(err) => shell.state.notify(err),
                            }
                            cx.notify();
                        })),
                )
        })
        .collect::<Vec<_>>();

    let body = if rows.is_empty() {
        div()
            .py(px(10.0))
            .text_size(px(11.0))
            .text_color(rgb(t.faint))
            .child(tr(lang, "还没有内容", "Nothing here yet"))
            .into_any_element()
    } else {
        div().flex().flex_col().children(rows).into_any_element()
    };

    div()
        .flex()
        .flex_col()
        .on_drop(cx.listener(move |shell, paths: &ExternalPaths, _, cx| {
            let Some(dir) = shell.state.current_instance_dir() else {
                return;
            };
            let mut imported = 0_usize;
            let mut last_error = None;
            for path in paths.paths() {
                match sealy_core::addons::import(&dir, kind, path) {
                    Ok(_) => imported += 1,
                    Err(err) => last_error = Some(err.to_string()),
                }
            }
            if imported > 0 {
                shell.state.notify(format!("已导入 {imported} 个文件"));
            } else if let Some(err) = last_error {
                shell.state.notify(err);
            }
            cx.notify();
        }))
        .child(
            div()
                .h(px(40.0))
                .flex()
                .items_center()
                .gap(px(10.0))
                .border_b_1()
                .border_color(rgb(t.line))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(title),
                )
                .child(mono(items.len().to_string(), t))
                .child(
                    quiet_button(tr(lang, "导入", "Import"), t)
                        .id(("addon-import", kind as usize))
                        .role(Role::Button)
                        .aria_label(format!("import-addons-{}", kind.folder()))
                        .on_click(cx.listener(move |_shell, _, _, cx| {
                            let receiver = cx.prompt_for_paths(PathPromptOptions {
                                files: true,
                                directories: true,
                                multiple: true,
                                prompt: Some("选择要导入的文件或文件夹".into()),
                            });
                            cx.spawn(async move |this, cx| {
                                let Ok(Ok(Some(paths))) = receiver.await else {
                                    return;
                                };
                                let _ = this.update(cx, |shell, cx| {
                                    match shell.state.import_addons(kind, &paths) {
                                        Ok(count) => {
                                            let lang = shell.state.language();
                                            shell.state.notify(format!(
                                                "{} {count}",
                                                tr(lang, "已导入", "Imported")
                                            ));
                                        }
                                        Err(err) => shell.state.notify(err),
                                    }
                                    cx.notify();
                                });
                            })
                            .detach();
                        })),
                ),
        )
        .child(body)
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
