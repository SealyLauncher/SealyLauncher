//! Settings: a sidebar with categories, and the controls for each category.

use gpui_kit::component::slider::Slider;
use sealy_core::config::{Language, MemoryMode, MotionSetting, SourcePolicy, ThemeName};

use crate::app::SealyShell;
use crate::components::{action_bar, choice_button, field_row, head_band_back, mono, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::{SettingsSection, ShellState};
use crate::theme::{self, ThemeTokens};

/// Width of the category sidebar.
const SIDEBAR_WIDTH: f32 = 208.0;
/// Width of the settings form.
const FORM_WIDTH: f32 = 720.0;

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();
    let section = state.settings_section;

    let content = match section {
        SettingsSection::Appearance => appearance_section(state, &t, lang, cx),
        SettingsSection::Downloads => downloads_section(state, &t, lang, cx),
        SettingsSection::JavaMemory => java_section(state, &t, lang, cx),
        SettingsSection::Network => network_section(state, &t, lang, cx),
        SettingsSection::Launcher => launcher_section(state, &t, lang, cx),
    };

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band_back(tr(lang, "设置", "Settings"), &t, cx))
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .child(sidebar(state, &t, lang, cx))
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.0))
                        .id("settings-body")
                        .overflow_y_scroll()
                        .p(px(theme::PAGE_PADDING))
                        .child(content),
                ),
        )
        .child(
            action_bar(&t).child(div().flex_1()).child(
                quiet_button(tr(lang, "打开数据目录", "Open data folder"), &t)
                    .id("open-data")
                    .role(Role::Button)
                    .aria_label("open-data")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.open_data_folder();
                        cx.notify();
                    })),
            ),
        )
        .into_any_element()
}

/// Category picker on the left; each entry pairs an icon with its label.
fn sidebar(
    state: &ShellState,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let current = state.settings_section;
    let accent = t.accent;
    let surface = t.surface;
    let bg = t.bg;
    let line = t.line;
    let line_soft = t.line_soft;
    let text = t.text;
    let muted = t.muted;

    let items = SettingsSection::ALL
        .into_iter()
        .enumerate()
        .map(|(index, section)| {
            let selected = section == current;
            let icon_tone = if selected { accent } else { muted };
            div()
                .id(("settings-section", index))
                .role(Role::Button)
                .aria_label(format!("settings-section-{index}"))
                .h(px(40.0))
                .w_full()
                .px(px(16.0))
                .flex()
                .items_center()
                .gap(px(10.0))
                .cursor_pointer()
                .border_l_1()
                .border_color(rgb(if selected { accent } else { surface }))
                .bg(rgb(if selected { surface } else { bg }))
                .hover(move |this| this.bg(rgb(line_soft)))
                .text_size(px(12.0))
                .text_color(rgb(if selected { accent } else { text }))
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.state.settings_section = section;
                    cx.notify();
                }))
                .child(
                    Icon::new(section_icon(section))
                        .w(px(15.0))
                        .h(px(15.0))
                        .text_color(rgb(icon_tone)),
                )
                .child(section.label(lang))
        })
        .collect::<Vec<_>>();

    div()
        .w(px(SIDEBAR_WIDTH))
        .flex_shrink_0()
        .h_full()
        .py(px(12.0))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .border_r_1()
        .border_color(rgb(line))
        .children(items)
        .into_any_element()
}

fn section_icon(section: SettingsSection) -> IconName {
    match section {
        SettingsSection::Appearance => IconName::Palette,
        SettingsSection::Downloads => IconName::Download,
        SettingsSection::JavaMemory => IconName::MemoryStick,
        SettingsSection::Network => IconName::Network,
        SettingsSection::Launcher => IconName::Info,
    }
}

/// Theme, scale, language, sound and motion.
fn appearance_section(
    state: &ShellState,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let swatches = ThemeName::ALL
        .into_iter()
        .map(|name| swatch(name, state.theme_name(), lang, t, cx))
        .collect::<Vec<_>>();

    let ui_scale_pct = state.settings.appearance.ui_scale_pct;
    let scale_buttons = [90_u16, 100, 115, 130]
        .into_iter()
        .map(|pct| {
            choice_button(format!("{pct}%"), ui_scale_pct == pct, t)
                .id(("ui-scale", usize::from(pct)))
                .role(Role::Button)
                .aria_label(format!("ui-scale-{pct}"))
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.state.set_ui_scale_pct(pct);
                    cx.notify();
                }))
        })
        .collect::<Vec<_>>();

    let music = state.settings.appearance.background_music;
    let motion = state.settings.motion;

    form(t)
        .child(section_title(tr(lang, "主题", "Theme"), t))
        .child(div().flex().gap(px(10.0)).children(swatches))
        .child(div().h(px(14.0)))
        .child(row(
            tr(lang, "界面缩放", "Interface scale"),
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .children(scale_buttons),
            t,
        ))
        .child(row(
            tr(lang, "语言", "Language"),
            div()
                .flex()
                .gap(px(8.0))
                .child(
                    choice_button("简体中文", lang == Language::ZhCn, t)
                        .id("lang-zh")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_language(Language::ZhCn);
                            cx.notify();
                        })),
                )
                .child(
                    choice_button("English", lang == Language::EnUs, t)
                        .id("lang-en")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_language(Language::EnUs);
                            cx.notify();
                        })),
                ),
            t,
        ))
        .child(row(
            tr(lang, "背景音乐", "Background music"),
            div()
                .flex()
                .gap(px(8.0))
                .child(
                    choice_button(tr(lang, "关", "Off"), !music, t)
                        .id("audio-off")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_audio(false);
                            cx.notify();
                        })),
                )
                .child(
                    choice_button(tr(lang, "开", "On"), music, t)
                        .id("audio-on")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_audio(true);
                            cx.notify();
                        })),
                ),
            t,
        ))
        .child(row(
            tr(lang, "动态效果", "Motion"),
            div()
                .flex()
                .gap(px(8.0))
                .child(motion_button(
                    tr(lang, "跟随系统", "System"),
                    MotionSetting::System,
                    motion,
                    t,
                    cx,
                ))
                .child(motion_button(
                    tr(lang, "始终", "Always"),
                    MotionSetting::Always,
                    motion,
                    t,
                    cx,
                ))
                .child(motion_button(
                    tr(lang, "从不", "Never"),
                    MotionSetting::Never,
                    motion,
                    t,
                    cx,
                )),
            t,
        ))
        .into_any_element()
}

/// Download source and mirror information.
fn downloads_section(
    state: &ShellState,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let source = state.settings.download.metadata_source;
    form(t)
        .child(row(
            tr(lang, "下载源", "Download source"),
            div()
                .flex()
                .gap(px(8.0))
                .child(source_button(
                    tr(lang, "自动", "Auto"),
                    SourcePolicy::Auto,
                    source,
                    t,
                    cx,
                ))
                .child(source_button(
                    tr(lang, "官方", "Official"),
                    SourcePolicy::Official,
                    source,
                    t,
                    cx,
                ))
                .child(source_button(
                    "BMCLAPI",
                    SourcePolicy::Bmclapi,
                    source,
                    t,
                    cx,
                )),
            t,
        ))
        .child(field_row(
            tr(lang, "BMCLAPI 镜像", "BMCLAPI mirror"),
            state.settings.download.bmclapi_root.clone(),
            t,
        ))
        .child(field_row(
            tr(lang, "并发下载", "Concurrent downloads"),
            format!("{}", state.settings.download.concurrency),
            t,
        ))
        .into_any_element()
}

/// Java runtime choice plus the heap controls.
fn java_section(
    state: &ShellState,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let java_default = state.settings.java.default_path.clone();
    let default_java = state
        .java
        .iter()
        .find(|java| Some(java.path.display().to_string()) == java_default)
        .or_else(|| state.java.first());
    let java_line = default_java
        .map(|java| format!("Java {} · {}", java.version, java.path.display()))
        .unwrap_or_else(|| tr(lang, "未检测到", "Not found").to_owned());
    let java_choices = state
        .java
        .iter()
        .take(4)
        .enumerate()
        .map(|(index, java)| {
            let path = java.path.display().to_string();
            let selected = Some(path.clone()) == java_default;
            choice_button(
                tr(lang, "Java ", "Java ").to_owned() + &java.major.to_string(),
                selected,
                t,
            )
            .id(("java-choice", index))
            .role(Role::Button)
            .aria_label(format!("java {}", java.major))
            .on_click(cx.listener(move |shell, _, _, cx| {
                shell
                    .state
                    .set_default_java(Some(std::path::Path::new(&path)));
                cx.notify();
            }))
        })
        .collect::<Vec<_>>();

    let java_row = if let Some(draft) = state.java_path_draft.clone() {
        row(
            tr(lang, "添加 Java", "Add Java"),
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .w(px(360.0))
                        .h(px(30.0))
                        .px(px(10.0))
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .rounded(px(theme::RADIUS_TILE))
                        .border_1()
                        .border_color(rgb(t.text))
                        .text_size(px(11.0))
                        .text_color(rgb(t.text))
                        .child(if draft.is_empty() {
                            tr(lang, "/path/to/java 然后回车", "/path/to/java, then Enter")
                                .to_owned()
                        } else {
                            draft
                        }),
                )
                .child(
                    quiet_button(tr(lang, "取消", "Cancel"), t)
                        .id("java-cancel")
                        .role(Role::Button)
                        .aria_label("java-cancel")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.java_path_draft = None;
                            cx.notify();
                        })),
                ),
            t,
        )
    } else {
        row(
            tr(lang, "添加 Java", "Add Java"),
            quiet_button(tr(lang, "输入路径", "Enter path"), t)
                .id("java-add")
                .role(Role::Button)
                .aria_label("java-add")
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.state.begin_add_java();
                    cx.notify();
                })),
            t,
        )
    };

    let auto = state.settings.java.memory_mode == MemoryMode::Auto;
    let memory = state.effective_memory_mb();
    let recommended = sealy_core::system::auto_memory_mb(state.total_ram_mb);
    let total_label = sealy_core::system::memory_label(state.total_ram_mb)
        .unwrap_or_else(|| tr(lang, "未知", "Unknown").to_owned());

    let mode_auto = choice_button(
        if state.total_ram_mb.is_some() {
            format!("{} · {} MB", tr(lang, "自动", "Auto"), recommended)
        } else {
            tr(lang, "自动", "Auto").to_owned()
        },
        auto,
        t,
    )
    .id("memory-auto")
    .role(Role::Button)
    .aria_label("memory-auto")
    .on_click(cx.listener(|shell, _, window, cx| {
        shell.state.set_memory_mode(MemoryMode::Auto);
        shell.sync_memory_slider(window, cx);
        cx.notify();
    }));
    let mode_manual = choice_button(tr(lang, "手动", "Manual"), !auto, t)
        .id("memory-manual")
        .role(Role::Button)
        .aria_label("memory-manual")
        .on_click(cx.listener(|shell, _, window, cx| {
            shell.state.set_memory_mode(MemoryMode::Manual);
            shell.sync_memory_slider(window, cx);
            cx.notify();
        }));

    let slider = state.memory_slider.as_ref().map(|slider| {
        let element = Slider::new(slider)
            .w(px(300.0))
            .bg(rgb(t.line))
            .text_color(rgb(t.text));
        if auto {
            element.disabled(true)
        } else {
            element
        }
    });

    let value_label = state
        .memory_draft
        .clone()
        .map(|draft| {
            if draft.is_empty() {
                "MB".to_owned()
            } else {
                format!("{draft} MB")
            }
        })
        .unwrap_or_else(|| format!("{memory} MB"));

    let common = [2048_u32, 4096, 8192]
        .into_iter()
        .map(|mb| {
            choice_button(
                format!("{mb} MB"),
                !auto && state.settings.java.memory.max_mb == mb,
                t,
            )
            .id(("memory-common", mb as usize))
            .role(Role::Button)
            .aria_label(format!("memory-common-{mb}"))
            .on_click(cx.listener(move |shell, _, window, cx| {
                shell.state.set_max_memory(mb);
                shell.sync_memory_slider(window, cx);
                cx.notify();
            }))
        })
        .collect::<Vec<_>>();

    let mut form = form(t)
        .child(row(
            tr(lang, "Java", "Java"),
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .children(java_choices)
                .child(
                    choice_button(tr(lang, "自动", "Auto"), java_default.is_none(), t)
                        .id("java-auto")
                        .role(Role::Button)
                        .aria_label("java-auto")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_default_java(None);
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "重新检测", "Rescan"), t)
                        .id("java-rescan")
                        .role(Role::Button)
                        .aria_label("java-rescan")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            let game = shell.state.paths.game.clone();
                            cx.spawn(async move |this, cx| {
                                let found = cx
                                    .background_executor()
                                    .spawn(async move { sealy_core::java::detect(&game) })
                                    .await;
                                let _ = this.update(cx, |shell, cx| {
                                    let count = found.len();
                                    shell.state.java = found;
                                    shell.state.notify(format!("已检测到 {count} 个 Java"));
                                    cx.notify();
                                });
                            })
                            .detach();
                        })),
                ),
            t,
        ))
        .child(field_row(
            tr(lang, "当前 Java", "Selected Java"),
            java_line,
            t,
        ))
        .child(java_row)
        .child(section_title(tr(lang, "游戏内存", "Game memory"), t))
        .child(row(
            tr(lang, "内存模式", "Memory mode"),
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(mode_auto)
                .child(mode_manual),
            t,
        ))
        .child(
            div()
                .h(px(64.0))
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(rgb(t.line_soft))
                .child(div().text_size(px(13.0)).text_color(rgb(t.text)).child(tr(
                    lang,
                    "最大内存",
                    "Maximum memory",
                )))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(div().w(px(300.0)).children(slider))
                        .child(
                            div()
                                .id("memory-input")
                                .role(Role::Button)
                                .aria_label("memory-input")
                                .h(px(30.0))
                                .w(px(96.0))
                                .px(px(10.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .overflow_hidden()
                                .rounded(px(theme::RADIUS_TILE))
                                .border_1()
                                .border_color(rgb(if state.memory_draft.is_some() {
                                    t.text
                                } else {
                                    t.line
                                }))
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(rgb(t.text))
                                .on_click(cx.listener(|shell, _, _, cx| {
                                    shell.state.begin_memory_draft();
                                    cx.notify();
                                }))
                                .child(value_label),
                        ),
                ),
        )
        .child(row(
            tr(lang, "常用值", "Presets"),
            div().flex().items_center().gap(px(8.0)).children(common),
            t,
        ))
        .child(field_row(
            tr(lang, "系统内存", "System memory"),
            if auto {
                format!(
                    "{total_label} · {}",
                    tr(lang, "由启动器决定", "chosen by launcher")
                )
            } else {
                format!(
                    "{total_label} · {}",
                    tr(lang, "可切换为自动", "switch to Auto")
                )
            },
            t,
        ));

    if state.memory_draft.is_some() {
        form = form.child(
            div()
                .h(px(28.0))
                .flex()
                .items_center()
                .text_size(px(11.0))
                .text_color(rgb(t.muted))
                .child(tr(
                    lang,
                    "输入数字后回车保存，Esc 取消（512–32768 MB）",
                    "Type a number then Enter; Esc cancels (512–32768 MB)",
                )),
        );
    }

    form.into_any_element()
}

/// Proxy and game folder.
fn network_section(
    state: &ShellState,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    form(t)
        .child(row(
            tr(lang, "HTTP 代理", "HTTP proxy"),
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .id("proxy-input")
                        .role(Role::Button)
                        .aria_label("proxy-input")
                        .h(px(30.0))
                        .w(px(260.0))
                        .px(px(10.0))
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .rounded(px(theme::RADIUS_TILE))
                        .border_1()
                        .border_color(rgb(if state.proxy_draft.is_some() {
                            t.text
                        } else {
                            t.line
                        }))
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(rgb(if state.proxy_draft.is_some() {
                            t.text
                        } else if state.settings.network.proxy.is_empty() {
                            t.faint
                        } else {
                            t.text
                        }))
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.proxy_draft =
                                Some(shell.state.settings.network.proxy.clone());
                            cx.notify();
                        }))
                        .child(state.proxy_draft.clone().unwrap_or_else(|| {
                            if state.settings.network.proxy.is_empty() {
                                tr(lang, "点击编辑代理", "Click to edit").to_owned()
                            } else {
                                state.settings.network.proxy.clone()
                            }
                        })),
                )
                .child(
                    quiet_button(tr(lang, "保存", "Save"), t)
                        .id("proxy-save")
                        .role(Role::Button)
                        .aria_label("proxy-save")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            let value = shell
                                .state
                                .proxy_draft
                                .clone()
                                .unwrap_or_else(|| shell.state.settings.network.proxy.clone());
                            shell.state.commit_proxy(value);
                            cx.notify();
                        })),
                ),
            t,
        ))
        .child(row(
            tr(lang, "游戏目录", "Game folder"),
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .id("game-dir-input")
                        .role(Role::Button)
                        .aria_label("game-dir-input")
                        .h(px(30.0))
                        .w(px(260.0))
                        .px(px(10.0))
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .rounded(px(theme::RADIUS_TILE))
                        .border_1()
                        .border_color(rgb(if state.game_dir_draft.is_some() {
                            t.text
                        } else {
                            t.line
                        }))
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(rgb(t.text))
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.game_dir_draft =
                                Some(shell.state.paths.game.display().to_string());
                            cx.notify();
                        }))
                        .child(
                            state
                                .game_dir_draft
                                .clone()
                                .unwrap_or_else(|| state.paths.game.display().to_string()),
                        ),
                )
                .child(
                    quiet_button(tr(lang, "切换", "Switch"), t)
                        .id("game-dir-save")
                        .role(Role::Button)
                        .aria_label("game-dir-save")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            let value =
                                shell.state.game_dir_draft.clone().unwrap_or_else(|| {
                                    shell.state.paths.game.display().to_string()
                                });
                            shell.state.commit_game_dir(value);
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "重置", "Reset"), t)
                        .id("game-dir-reset")
                        .role(Role::Button)
                        .aria_label("game-dir-reset")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.commit_game_dir(String::new());
                            cx.notify();
                        })),
                ),
            t,
        ))
        .child(field_row(
            tr(lang, "数据目录", "Data directory"),
            state.paths.root.display().to_string(),
            t,
        ))
        .into_any_element()
}

/// Update, version and maintenance.
fn launcher_section(
    state: &ShellState,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    form(t)
        .child(row(
            tr(lang, "启动时检查更新", "Check on startup"),
            div()
                .flex()
                .gap(px(8.0))
                .child(
                    choice_button(tr(lang, "开", "On"), state.settings.update.auto_check, t)
                        .id("auto-update-on")
                        .role(Role::Button)
                        .aria_label("auto-update-on")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_auto_check_update(true);
                            cx.notify();
                        })),
                )
                .child(
                    choice_button(tr(lang, "关", "Off"), !state.settings.update.auto_check, t)
                        .id("auto-update-off")
                        .role(Role::Button)
                        .aria_label("auto-update-off")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_auto_check_update(false);
                            cx.notify();
                        })),
                ),
            t,
        ))
        .child(row(
            tr(lang, "启动器更新", "Launcher update"),
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(mono(format!("v{}", sealy_core::version()), t))
                .child(
                    quiet_button(tr(lang, "检查更新", "Check"), t)
                        .id("check-update")
                        .role(Role::Button)
                        .aria_label("check-update")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.start_update_check(cx);
                            cx.notify();
                        })),
                )
                .child(update_status(state, lang, t, cx)),
            t,
        ))
        .child(field_row(
            tr(lang, "版本", "Version"),
            format!("{} · portable", sealy_core::version()),
            t,
        ))
        .child(field_row(
            tr(lang, "Microsoft 客户端 ID", "Microsoft client ID"),
            state.settings.auth.microsoft_client_id.clone(),
            t,
        ))
        .child(row(
            tr(lang, "调试日志", "Debug logging"),
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(div().text_size(px(11.0)).text_color(rgb(t.muted)).child(tr(
                    lang,
                    "开启后把 DEBUG 级别写入 SealyData/logs",
                    "Writes DEBUG lines to SealyData/logs",
                )))
                .child(
                    quiet_button(tr(lang, "打开日志", "Open logs"), t)
                        .id("open-logs")
                        .role(Role::Button)
                        .aria_label("open-logs")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.open_logs_folder();
                            cx.notify();
                        })),
                )
                .child(
                    choice_button(
                        tr(lang, "开", "On"),
                        state.settings.debug.verbose_logging,
                        t,
                    )
                    .id("debug-on")
                    .role(Role::Button)
                    .aria_label("debug-on")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.set_verbose_logging(true);
                        cx.notify();
                    })),
                )
                .child(
                    choice_button(
                        tr(lang, "关", "Off"),
                        !state.settings.debug.verbose_logging,
                        t,
                    )
                    .id("debug-off")
                    .role(Role::Button)
                    .aria_label("debug-off")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.set_verbose_logging(false);
                        cx.notify();
                    })),
                ),
            t,
        ))
        .child(row(
            tr(lang, "设置向导", "Setup wizard"),
            quiet_button(tr(lang, "重新运行", "Run again"), t)
                .id("restart-oobe")
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.state.start_oobe();
                    cx.notify();
                })),
            t,
        ))
        .into_any_element()
}

/// The shared column container for one category.
fn form(t: &ThemeTokens) -> Div {
    div()
        .w(px(FORM_WIDTH))
        .flex()
        .flex_col()
        .border_t_1()
        .border_color(rgb(t.line))
}

fn section_title(label: &'static str, t: &ThemeTokens) -> Div {
    div()
        .h(px(44.0))
        .flex()
        .items_center()
        .text_size(px(13.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(t.text))
        .child(label)
}

fn row(label: &'static str, control: impl IntoElement, t: &ThemeTokens) -> Div {
    div()
        .h(px(52.0))
        .flex()
        .items_center()
        .justify_between()
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .child(
            div()
                .text_size(px(13.0))
                .text_color(rgb(t.text))
                .child(label),
        )
        .child(control)
}

fn source_button(
    label: &'static str,
    value: SourcePolicy,
    current: SourcePolicy,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    choice_button(label, value == current, t)
        .id(match value {
            SourcePolicy::Auto => "source-auto",
            SourcePolicy::Official => "source-official",
            SourcePolicy::Bmclapi => "source-bmclapi",
        })
        .on_click(cx.listener(move |shell, _, _, cx| {
            shell.state.set_source(value);
            cx.notify();
        }))
        .into_any_element()
}

fn motion_button(
    label: &'static str,
    value: MotionSetting,
    current: MotionSetting,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    choice_button(label, value == current, t)
        .id(match value {
            MotionSetting::System => "motion-system",
            MotionSetting::Always => "motion-always",
            MotionSetting::Never => "motion-never",
        })
        .on_click(cx.listener(move |shell, _, _, cx| {
            shell.state.set_motion(value);
            cx.notify();
        }))
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

/// Status text and download button for the self-update row.
fn update_status(
    state: &ShellState,
    lang: Language,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    if state.update_downloading {
        let (done, total) = state
            .update_progress
            .as_ref()
            .map(|progress| {
                (
                    progress.done.load(std::sync::atomic::Ordering::Relaxed),
                    progress.total,
                )
            })
            .unwrap_or((0, 0));
        let ratio = if total > 0 {
            (done as f32 / total as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let label = format!(
            "{:.1} / {:.1} MB",
            done as f64 / 1_048_576.0,
            total as f64 / 1_048_576.0
        );
        return div()
            .flex()
            .items_center()
            .gap(px(10.0))
            .child(
                div()
                    .w(px(150.0))
                    .h(px(4.0))
                    .rounded(px(2.0))
                    .bg(rgb(t.line_soft))
                    .child(
                        div()
                            .h(px(4.0))
                            .w(px(150.0 * ratio))
                            .rounded(px(2.0))
                            .bg(rgb(t.text)),
                    ),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(t.muted))
                    .child(label),
            )
            .child(
                quiet_button(tr(lang, "取消", "Cancel"), t)
                    .id("cancel-update")
                    .role(Role::Button)
                    .aria_label("cancel-update")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.cancel_update();
                        cx.notify();
                    })),
            )
            .into_any_element();
    }

    if let Some(error) = &state.update_error {
        let release = match &state.update_check {
            Some(Ok(Some(release))) => Some(release.clone()),
            _ => None,
        };
        let retry = release.map(|release| {
            quiet_button(tr(lang, "重试", "Retry"), t)
                .id("retry-update")
                .role(Role::Button)
                .aria_label("retry-update")
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.state.download_update(&release, cx);
                    cx.notify();
                }))
        });
        return div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                div()
                    .max_w(px(220.0))
                    .overflow_hidden()
                    .text_size(px(11.0))
                    .text_color(rgb(t.danger))
                    .child(error.clone()),
            )
            .children(retry)
            .into_any_element();
    }

    match &state.update_check {
        None => div().into_any_element(),
        Some(Err(err)) => div()
            .text_size(px(11.0))
            .text_color(rgb(t.muted))
            .child(err.clone())
            .into_any_element(),
        Some(Ok(None)) => div()
            .text_size(px(11.0))
            .text_color(rgb(t.muted))
            .child(tr(lang, "已是最新版本", "Up to date"))
            .into_any_element(),
        Some(Ok(Some(release))) => {
            let release = release.clone();
            let has_asset = release.asset.is_some();
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(rgb(t.text))
                        .child(format!("→ v{}", release.version)),
                )
                .child(
                    quiet_button(tr(lang, "下载并安装", "Download and install"), t)
                        .id("download-update")
                        .role(Role::Button)
                        .aria_label("download-update")
                        .when(!has_asset, |this| this.opacity(0.5))
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            shell.state.download_update(&release, cx);
                            cx.notify();
                        })),
                )
                .into_any_element()
        }
    }
}
