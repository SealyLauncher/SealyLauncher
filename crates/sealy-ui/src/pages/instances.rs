//! Instances: the launchable profiles and their own game directories.

use sealy_core::instances::Instance;

use crate::app::SealyShell;
use crate::components::{
    action_bar, head_band_back, icon_button, mono, primary_button, quiet_button,
};
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
        .enumerate()
        .map(|(index, instance)| row(index, instance, state, &t, cx))
        .collect::<Vec<_>>();

    let body = if state.rename_draft.is_some() {
        rename_card(state, &t, lang, cx)
    } else if state.new_instance_open {
        create_panel(state, &t, lang, cx)
    } else if rows.is_empty() {
        empty(&t, lang)
    } else {
        div()
            .id("instance-list")
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scrollbar()
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
        .relative()
        .flex()
        .flex_col()
        .child(head_band_back(tr(lang, "实例", "Instances"), &t, cx))
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
                    icon_button(IconName::Pencil, &t)
                        .id("instance-rename")
                        .role(Role::Button)
                        .aria_label("rename-instance")
                        .on_click(cx.listener(|shell, _, window, cx| {
                            shell.state.begin_rename();
                            let name = shell
                                .state
                                .instances
                                .current()
                                .map(|instance| instance.name.clone())
                                .unwrap_or_default();
                            shell.open_input(crate::inputs::InputField::Rename, name, window, cx);
                        })),
                )
                .child(
                    icon_button(IconName::Copy, &t)
                        .id("instance-copy")
                        .role(Role::Button)
                        .aria_label("copy-instance")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.duplicate_instance();
                            cx.notify();
                        })),
                )
                .child(
                    icon_button(IconName::Logs, &t)
                        .id("instance-logs")
                        .role(Role::Button)
                        .aria_label("open-instance-logs")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.open_logs();
                            cx.notify();
                        })),
                )
                .child(
                    icon_button(IconName::Settings2, &t)
                        .id("instance-settings")
                        .role(Role::Button)
                        .aria_label("instance-launch-settings")
                        .on_click(cx.listener(|shell, _, window, cx| {
                            shell.state.begin_instance_settings();
                            shell.load_launch_inputs(window, cx);
                        })),
                )
                .child(
                    icon_button(IconName::FolderOpen, &t)
                        .id("instance-folder")
                        .role(Role::Button)
                        .aria_label("open-instance-folder")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.open_instance_folder();
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "删除", "Delete"), &t)
                        .id("instance-delete")
                        .role(Role::Button)
                        .aria_label("delete-instance")
                        .on_click(cx.listener(|shell, _, window, cx| {
                            shell.open_delete_dialog(window, cx);
                        })),
                )
                .child(
                    quiet_button(tr(lang, "导入整合包", "Import modpack"), &t)
                        .id("import-modpack")
                        .role(Role::Button)
                        .aria_label("import-modpack")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            // Debug builds accept a pack path from the environment
                            // so the import flow can be tested without the native
                            // file dialog.
                            #[cfg(debug_assertions)]
                            if let Ok(path) = std::env::var("SEALY_DEBUG_IMPORT_PACK") {
                                crate::install_flow::start_modpack(
                                    &mut shell.state,
                                    std::path::PathBuf::from(path),
                                    cx,
                                );
                                return;
                            }
                            let receiver = cx.prompt_for_paths(PathPromptOptions {
                                files: true,
                                directories: false,
                                multiple: false,
                                prompt: Some("选择整合包（.mrpack / .zip）".into()),
                            });
                            cx.spawn(async move |this, cx| {
                                let Ok(Ok(Some(paths))) = receiver.await else {
                                    return;
                                };
                                if let Some(path) = paths.into_iter().next() {
                                    let _ = this.update(cx, |shell, cx| {
                                        crate::install_flow::start_modpack(
                                            &mut shell.state,
                                            path,
                                            cx,
                                        );
                                    });
                                }
                            })
                            .detach();
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

/// Per-instance launch settings sheet (window size, memory, JVM args).
pub(crate) fn launch_settings_sheet(
    sheet: Sheet,
    state: &ShellState,
    cx: &mut Context<SealyShell>,
) -> Sheet {
    if state.inst_launch_draft.is_none() {
        return sheet;
    }
    let t = state.theme();
    let lang = state.language();

    let fields = (0..4)
        .map(|index| {
            let field = match index {
                0 => crate::inputs::InputField::LaunchWidth,
                1 => crate::inputs::InputField::LaunchHeight,
                2 => crate::inputs::InputField::LaunchMemory,
                _ => crate::inputs::InputField::LaunchJvm,
            };
            div()
                .h(px(46.0))
                .flex()
                .items_center()
                .gap(px(12.0))
                .border_b_1()
                .border_color(rgb(t.line_soft))
                .child(
                    div()
                        .w(px(120.0))
                        .text_size(px(12.0))
                        .text_color(rgb(t.muted))
                        .child(match index {
                            0 => tr(lang, "窗口宽度", "Width"),
                            1 => tr(lang, "窗口高度", "Height"),
                            2 => tr(lang, "最大内存 (MB)", "Max memory (MB)"),
                            _ => tr(lang, "JVM 参数", "JVM arguments"),
                        }),
                )
                .child(crate::inputs::field(state, field, 360.0, &t))
        })
        .collect::<Vec<_>>();

    sheet
        .size(DefiniteLength::Absolute(px(560.0).into()))
        .bg(rgb(t.surface))
        .border_color(rgb(t.line))
        .title(div().text_color(rgb(t.text)).child(tr(
            lang,
            "实例启动设置",
            "Instance launch settings",
        )))
        .child(
            div()
                .pb(px(6.0))
                .text_size(px(11.0))
                .text_color(rgb(t.faint))
                .child(tr(
                    lang,
                    "留空则跟随全局设置 · 点击字段编辑 · Enter 保存 · Esc 取消",
                    "Empty follows global settings · Click a field · Enter save · Esc cancel",
                )),
        )
        .child(div().w_full().children(fields))
        .footer(
            action_bar(&t)
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "取消", "Cancel"), &t)
                        .id("instance-settings-cancel")
                        .role(Role::Button)
                        .aria_label("cancel-instance-settings")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.inst_launch_draft = None;
                            shell.state.instance_settings_open = false;
                            cx.notify();
                        })),
                )
                .child(
                    primary_button(tr(lang, "保存", "Save"), &t)
                        .id("instance-settings-save")
                        .role(Role::Button)
                        .aria_label("save-instance-settings")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            if let Err(err) = shell.state.commit_instance_settings() {
                                shell.state.notify(err);
                            }
                            cx.notify();
                        })),
                ),
        )
}
/// Read-only viewer for the selected instance's latest game log.
pub(crate) fn logs_sheet(sheet: Sheet, state: &ShellState, cx: &mut Context<SealyShell>) -> Sheet {
    let t = state.theme();
    let lang = state.language();
    let count = state.logs_lines.len();
    let lines = state
        .logs_lines
        .iter()
        .map(|line| {
            div()
                .flex_shrink_0()
                .text_size(px(10.5))
                .line_height(relative(1.35))
                .text_color(rgb(t.muted))
                .child(line.clone())
        })
        .collect::<Vec<_>>();

    let body: AnyElement = if lines.is_empty() {
        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(8.0))
            .py(px(48.0))
            .child(
                Icon::new(IconName::ScrollText)
                    .w(px(24.0))
                    .h(px(24.0))
                    .text_color(rgb(t.faint)),
            )
            .child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
                lang,
                "还没有日志，启动一次游戏后再来看",
                "No log yet — launch the game once",
            )))
            .into_any_element()
    } else {
        div()
            .id("instance-log-lines")
            .w_full()
            .flex()
            .flex_col()
            .gap(px(1.0))
            .children(lines)
            .into_any_element()
    };

    sheet
        .size(DefiniteLength::Absolute(px(720.0).into()))
        .bg(rgb(t.surface))
        .border_color(rgb(t.line))
        .title(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(div().text_color(rgb(t.text)).child(tr(
                            lang,
                            "游戏日志",
                            "Game log",
                        )))
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(t.faint))
                                .child(state.logs_title.clone()),
                        ),
                )
                .child(mono(format!("{} 行", count), &t)),
        )
        .child(body)
        .footer(
            action_bar(&t)
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "刷新", "Refresh"), &t)
                        .id("instance-logs-refresh")
                        .role(Role::Button)
                        .aria_label("refresh-instance-logs")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.refresh_logs();
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "打开日志目录", "Open log folder"), &t)
                        .id("instance-logs-folder")
                        .role(Role::Button)
                        .aria_label("open-log-folder")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.open_log_folder();
                            cx.notify();
                        })),
                ),
        )
}
/// Inline editor for renaming the selected instance.
fn rename_card(
    state: &ShellState,
    t: &ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(14.0))
        .child(crate::inputs::field(
            state,
            crate::inputs::InputField::Rename,
            420.0,
            t,
        ))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(div().text_size(px(11.0)).text_color(rgb(t.muted)).child(tr(
                    lang,
                    "回车确认 · Esc 取消",
                    "Enter to confirm · Esc to cancel",
                )))
                .child(
                    quiet_button(tr(lang, "取消", "Cancel"), t)
                        .id("rename-cancel")
                        .role(Role::Button)
                        .aria_label("rename-cancel")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.cancel_rename();
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "确认", "Confirm"), t)
                        .id("rename-confirm")
                        .role(Role::Button)
                        .aria_label("rename-confirm")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.commit_rename();
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}
/// New-instance panel: a scrollable version list with loader actions.
fn create_panel(
    state: &ShellState,
    t: &ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let picker = &state.new_instance;
    let chosen_game = picker.game.clone();

    let mut list = div()
        .id("create-list")
        .flex_1()
        .min_h(px(0.0))
        .w_full()
        .overflow_y_scrollbar()
        .flex()
        .flex_col();

    if state.installed.is_empty() {
        list = list.child(
            div()
                .py(px(18.0))
                .text_size(px(12.0))
                .text_color(rgb(t.muted))
                .child(tr(
                    lang,
                    "还没有已安装的版本，先去下载页安装一个",
                    "No installed version — install one from Downloads",
                )),
        );
    } else {
        // Base versions first: those are the usual starting points for a new
        // instance, loader builds follow with their parent in the subtitle.
        let mut ordered: Vec<&sealy_core::version::InstalledVersion> =
            state.installed.iter().collect();
        ordered.sort_by(|left, right| {
            let left_is_base = version_parent(&left.document).is_none();
            let right_is_base = version_parent(&right.document).is_none();
            right_is_base
                .cmp(&left_is_base)
                .then_with(|| left.id.cmp(&right.id))
        });
        for (index, version) in ordered.into_iter().take(60).enumerate() {
            let id = version.id.clone();
            let (kind, inherits) = version_meta(&version.document, lang);
            let subtitle = match &inherits {
                Some(parent) => format!("{kind} · {parent}"),
                None => kind.clone(),
            };
            let selected = chosen_game.as_deref() == Some(id.as_str());

            let pick = {
                let id = id.clone();
                cx.listener(move |shell, _, _, cx| {
                    let changed = shell.state.new_instance.game.as_deref() != Some(id.as_str());
                    shell.state.pick_new_instance_game(&id);
                    if changed
                        && let Some(loader) = shell.state.new_instance.loader
                        && !loader.needs_local_installer()
                    {
                        fetch_loader_choices(cx, loader, id.clone());
                    }
                    cx.notify();
                })
            };
            let create_vanilla = {
                let id = id.clone();
                cx.listener(move |shell, _, _, cx| {
                    shell.state.pick_new_instance_game(&id);
                    shell.state.new_instance.loader = None;
                    shell.state.create_instance(&id);
                    shell.state.new_instance_open = false;
                    cx.notify();
                })
            };

            list = list.child(
                div()
                    .id(("create-row", index))
                    .role(Role::Button)
                    .aria_label(format!("pick {id}"))
                    .h(px(56.0))
                    .w_full()
                    .flex_shrink_0()
                    .px(px(8.0))
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .border_1()
                    .border_color(rgb(if selected { t.accent } else { t.line_soft }))
                    .cursor_pointer()
                    .hover({
                        let hover = t.line_soft;
                        move |this| this.bg(rgb(hover))
                    })
                    .on_click(pick)
                    .child(
                        div()
                            .w(px(26.0))
                            .h(px(26.0))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(theme::RADIUS_TILE))
                            .border_1()
                            .border_color(rgb(t.line))
                            .child(
                                Icon::new(IconName::Box)
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .text_color(rgb(t.text)),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(if selected { t.accent } else { t.text }))
                                    .child(id.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .text_color(rgb(t.muted))
                                    .child(subtitle),
                            ),
                    )
                    .when(selected, |this| {
                        this.child(
                            quiet_button(tr(lang, "直接新建", "Create"), t)
                                .id(("create-vanilla", index))
                                .role(Role::Button)
                                .aria_label(format!("create-vanilla {id}"))
                                .on_click(create_vanilla),
                        )
                    }),
            );
        }
    }

    let loader_section = match &chosen_game {
        Some(game) => loader_section(state, game, t, lang, cx),
        None => div()
            .text_size(px(11.0))
            .text_color(rgb(t.muted))
            .child(tr(
                lang,
                "先在上面选择要基于哪个版本",
                "Pick a version above first",
            ))
            .into_any_element(),
    };

    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .pt(px(6.0))
        .child(
            div()
                .flex()
                .items_baseline()
                .gap(px(10.0))
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(tr(lang, "选择版本", "Choose a version")),
                )
                .child(div().text_size(px(11.0)).text_color(rgb(t.muted)).child(tr(
                    lang,
                    "点一次选中，再在下方选择加载器",
                    "Pick one, then choose a loader below",
                ))),
        )
        .child(div().max_h(px(210.0)).flex().child(list))
        .child(loader_section)
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    quiet_button(tr(lang, "取消", "Cancel"), t)
                        .id("pick-cancel")
                        .role(Role::Button)
                        .aria_label("pick-cancel")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.new_instance_open = false;
                            cx.notify();
                        })),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(rgb(t.muted))
                        .child(format!(
                            "{} {}",
                            state.installed.len(),
                            tr(lang, "个已安装版本", "installed versions")
                        )),
                ),
        )
        .into_any_element()
}

/// Loader choices for the selected version plus the create button.
fn loader_section(
    state: &ShellState,
    game: &str,
    t: &ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let picker = &state.new_instance;
    let current = picker.loader;

    let mut chips = Vec::new();
    chips.push(loader_chip(
        "loader-vanilla",
        tr(lang, "原版", "Vanilla"),
        current.is_none(),
        t,
        cx.listener(|shell, _, _, cx| {
            shell.state.set_new_instance_loader(None);
            cx.notify();
        }),
    ));
    for kind in sealy_core::loader::LoaderKind::ALL {
        let game = game.to_owned();
        chips.push(loader_chip(
            ("loader-kind", kind as usize),
            kind.label(),
            current == Some(kind),
            t,
            cx.listener(move |shell, _, _, cx| {
                shell.state.set_new_instance_loader(Some(kind));
                if !kind.needs_local_installer() {
                    fetch_loader_choices(cx, kind, game.clone());
                }
                cx.notify();
            }),
        ));
    }

    let body: AnyElement = match current {
        None => div()
            .text_size(px(11.0))
            .text_color(rgb(t.muted))
            .child(tr(
                lang,
                "创建纯原版实例，不安装加载器",
                "Create a vanilla instance without a loader",
            ))
            .into_any_element(),
        Some(kind) if kind.needs_local_installer() => {
            let chosen = picker
                .optifine_jar
                .as_ref()
                .map(|jar| jar.display().to_string())
                .unwrap_or_default();
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    quiet_button(tr(lang, "选择安装包", "Choose installer"), t)
                        .id("pick-optifine")
                        .role(Role::Button)
                        .aria_label("pick-optifine")
                        .on_click(cx.listener(|_shell, _, _, cx| {
                            let receiver = cx.prompt_for_paths(PathPromptOptions {
                                files: true,
                                directories: false,
                                multiple: false,
                                prompt: Some("选择 OptiFine 安装包".into()),
                            });
                            cx.spawn(async move |this, cx| {
                                if let Ok(Ok(Some(paths))) = receiver.await
                                    && let Some(path) = paths.into_iter().next()
                                {
                                    let _ = this.update(cx, |shell, cx| {
                                        shell.state.new_instance.optifine_jar = Some(path);
                                        cx.notify();
                                    });
                                }
                            })
                            .detach();
                        })),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .text_size(px(10.0))
                        .text_color(rgb(if chosen.is_empty() { t.faint } else { t.text }))
                        .child(if chosen.is_empty() {
                            tr(
                                lang,
                                "需要本地 OptiFine 安装包",
                                "A local OptiFine installer is required",
                            )
                            .to_owned()
                        } else {
                            chosen
                        }),
                )
                .into_any_element()
        }
        Some(_) if picker.loader_loading => div()
            .text_size(px(11.0))
            .text_color(rgb(t.muted))
            .child(tr(lang, "正在获取版本列表…", "Loading versions…"))
            .into_any_element(),
        Some(kind) => {
            let mut entries = Vec::new();
            entries.push(loader_chip(
                "loader-version-latest",
                tr(lang, "最新", "Latest"),
                picker.loader_version.is_none(),
                t,
                cx.listener(|shell, _, _, cx| {
                    shell.state.new_instance.loader_version = None;
                    cx.notify();
                }),
            ));
            for (index, choice) in picker.loader_choices.iter().take(10).enumerate() {
                let version = choice.version.clone();
                let stable = choice.stable;
                let selected = picker.loader_version.as_deref() == Some(version.as_str());
                entries.push(loader_chip(
                    ("loader-version", index),
                    if stable {
                        version.clone()
                    } else {
                        format!("{version} *")
                    },
                    selected,
                    t,
                    cx.listener(move |shell, _, _, cx| {
                        shell.state.new_instance.loader_version = Some(version.clone());
                        cx.notify();
                    }),
                ));
                let _ = index;
            }
            let hint = match &picker.loader_error {
                Some(err) => err.clone(),
                None => format!("{} · {game}", kind.label()),
            };
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(div().flex().flex_wrap().gap(px(6.0)).children(entries))
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(rgb(t.muted))
                        .child(hint),
                )
                .into_any_element()
        }
    };

    let can_create = match current {
        None => true,
        Some(kind) if kind.needs_local_installer() => picker.optifine_jar.is_some(),
        Some(_) => true,
    };
    let create_label = match current {
        None => tr(lang, "创建实例", "Create instance").to_owned(),
        Some(kind) => format!("{} {}", tr(lang, "安装", "Install"), kind.label()),
    };

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(div().h(px(1.0)).w_full().bg(rgb(t.line)))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(tr(lang, "加载器", "Loader")),
                )
                .children(chips),
        )
        .child(body)
        .child(
            div().flex().justify_end().child(
                primary_button(create_label, t)
                    .id("create-confirm")
                    .role(Role::Button)
                    .aria_label("create-confirm")
                    .when(!can_create, |this| this.opacity(0.5))
                    .on_click(cx.listener(move |shell, _, _, cx| {
                        let Some(game) = shell.state.new_instance.game.clone() else {
                            shell.state.notify(tr(
                                shell.state.language(),
                                "请先选择版本",
                                "Pick a version first",
                            ));
                            return;
                        };
                        let loader = shell.state.new_instance.loader;
                        match loader {
                            None => {
                                shell.state.create_instance(&game);
                                shell.state.new_instance_open = false;
                                cx.notify();
                            }
                            Some(kind) => {
                                let jar = shell.state.new_instance.optifine_jar.clone();
                                if kind.needs_local_installer() && jar.is_none() {
                                    shell.state.notify(tr(
                                        shell.state.language(),
                                        "请先选择 OptiFine 安装包",
                                        "Choose the OptiFine installer first",
                                    ));
                                    return;
                                }
                                let request = crate::install_flow::LoaderRequest {
                                    game,
                                    kind,
                                    version: shell.state.new_instance.loader_version.clone(),
                                    jar,
                                    instance_name: Some("new".to_owned()),
                                };
                                shell.state.new_instance_open = false;
                                crate::install_flow::start_loader_request(
                                    &mut shell.state,
                                    request,
                                    cx,
                                );
                            }
                        }
                    })),
            ),
        )
        .into_any_element()
}

/// Small selectable chip used by the loader pickers.
fn loader_chip(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    t: &ThemeTokens,
    listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let label: SharedString = label.into();
    div()
        .id(id)
        .role(Role::Button)
        .aria_label(label.clone())
        .h(px(26.0))
        .px(px(10.0))
        .flex()
        .items_center()
        .rounded(px(theme::RADIUS_TILE))
        .border_1()
        .border_color(rgb(if selected { t.accent } else { t.line }))
        .cursor_pointer()
        .text_size(px(11.0))
        .text_color(rgb(if selected { t.accent } else { t.text }))
        .on_click(listener)
        .child(label)
        .into_any_element()
}

/// Fetches the loader builds for `game` in the background.
fn fetch_loader_choices(
    cx: &mut Context<SealyShell>,
    kind: sealy_core::loader::LoaderKind,
    game: String,
) {
    cx.spawn(async move |this, cx| {
        let _ = this.update(cx, |shell, cx| {
            shell.state.new_instance.loader_loading = true;
            shell.state.new_instance.loader_error = None;
            cx.notify();
        });
        let result = cx
            .background_executor()
            .spawn({
                let game = game.clone();
                async move { sealy_core::loader::versions(kind, &game) }
            })
            .await;
        let _ = this.update(cx, |shell, cx| {
            shell.state.new_instance.loader_loading = false;
            match result {
                Ok(choices) => {
                    if shell.state.new_instance.loader == Some(kind) {
                        shell.state.new_instance.loader_choices = choices;
                    }
                }
                Err(err) => {
                    shell.state.new_instance.loader_error = Some(err.to_string());
                }
            }
            cx.notify();
        });
    })
    .detach();
}
/// Parent version id of an installed document, if it inherits one.
fn version_parent(document: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(document).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&text).ok()?;
    json["inheritsFrom"].as_str().map(str::to_owned)
}
/// Reads the version type and parent id from an installed document.
fn version_meta(
    document: &std::path::Path,
    lang: sealy_core::config::Language,
) -> (String, Option<String>) {
    let Ok(text) = std::fs::read_to_string(document) else {
        return (tr(lang, "未知", "unknown").to_owned(), None);
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return (tr(lang, "未知", "unknown").to_owned(), None);
    };

    let kind = json["type"].as_str().unwrap_or("release").to_owned();
    let parent = json["inheritsFrom"].as_str().map(str::to_owned);
    (kind, parent)
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
    index: usize,
    instance: &Instance,
    state: &ShellState,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let selected = state.instances.selected.as_deref() == Some(instance.name.as_str());
    let name = instance.name.clone();
    let shown_name = instance.name.clone();
    let meta = instance.version_id.clone();
    let icon = instance_icon(instance);
    let played = instance
        .last_played
        .map(age_label)
        .unwrap_or_else(|| "—".to_owned());

    div()
        .id(("instance", index))
        .role(Role::Button)
        .aria_label(instance.name.clone())
        .h(px(72.0))
        .flex_shrink_0()
        .px(px(12.0))
        .flex()
        .items_center()
        .gap(px(14.0))
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .cursor_pointer()
        .when(selected, |this| {
            let accent = t.accent;
            this.bg(rgb(t.surface)).border_1().border_color(rgb(accent))
        })
        .hover(move |this| this.bg(rgb(t.surface)))
        .when(selected, |this| {
            let accent = t.accent;
            this.child(
                div()
                    .w(px(3.0))
                    .h(px(44.0))
                    .flex_shrink_0()
                    .rounded(px(2.0))
                    .bg(rgb(accent)),
            )
        })
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
                    Icon::new(icon)
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
                        .text_color(rgb(if selected { t.accent } else { t.text }))
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
            if shell.state.register_instance_click(&name) {
                shell.state.open_instance_detail();
            }
            cx.notify();
        }))
        .into_any_element()
}

/// Maps an instance's stored icon key to a built-in icon.
pub(crate) fn instance_icon(instance: &Instance) -> IconName {
    match instance.icon.as_deref() {
        Some("layers") => IconName::Layers,
        Some("package") => IconName::Package,
        Some("server") => IconName::Server,
        _ => IconName::Box,
    }
}

/// Confirmation panel shown before deleting an instance.
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
