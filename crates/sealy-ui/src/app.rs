//! Application shell: window chrome, page routing and transient notices.

use gpui_kit::component::TitleBar;

use crate::i18n::tr;
use crate::kit::*;
use crate::pages;
use crate::state::{Route, ShellState};
use crate::theme;

/// Starts the GPUI application and opens the shell window.
pub fn run(bootstrap: sealy_core::Bootstrap) {
    let bootstrap = std::rc::Rc::new(bootstrap);
    let window = std::rc::Rc::new(std::cell::RefCell::new(None::<AnyWindowHandle>));

    let reopen_window = window.clone();
    let reopen_bootstrap = bootstrap.clone();
    let application = gpui_kit::application().with_assets(crate::assets::SealyAssets);
    application.on_reopen(move |cx| {
        // macOS calls this when the dock icon is clicked without windows.
        if let Some(handle) = *reopen_window.borrow()
            && handle
                .update(cx, |_, window, _| window.activate_window())
                .is_ok()
        {
            cx.activate(true);
            return;
        }
        match open_shell_window(reopen_bootstrap.clone(), cx) {
            Ok(handle) => {
                *reopen_window.borrow_mut() = Some(handle);
                cx.activate(true);
            }
            Err(err) => eprintln!("failed to reopen the window: {err}"),
        }
    });
    application.run(move |cx| {
        gpui_kit::init(cx);
        crate::kit::set_ui_scale(bootstrap.settings.appearance.ui_scale_pct as f32 / 100.0);
        crate::menu::install(cx, bootstrap.settings.language);

        let opened = open_shell_window(bootstrap.clone(), cx)
            .expect("failed to open the SealyLauncher window");
        *window.borrow_mut() = Some(opened);
    });
}

/// Opens the shell window and returns its handle.
fn open_shell_window(
    bootstrap: std::rc::Rc<sealy_core::Bootstrap>,
    cx: &mut App,
) -> gpui_kit::Result<AnyWindowHandle> {
    let bounds = Bounds::centered(
        None,
        size(px(theme::WINDOW_WIDTH), px(theme::WINDOW_HEIGHT)),
        cx,
    );
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(theme::MIN_WIDTH), px(theme::MIN_HEIGHT))),
        ..TitleBar::window_options()
    };

    let (handle, _root) = gpui_kit::open_window(options, cx, move |window, cx| {
        let bootstrap = bootstrap.clone();
        cx.new(|cx| SealyShell::new(bootstrap, window, cx))
    })?;
    Ok(handle)
}

pub(crate) struct SealyShell {
    pub(crate) state: ShellState,
    focus: FocusHandle,
    _memory_subscription: Subscription,
    _input_subscriptions: Vec<Subscription>,
    /// Download-source dropdown selection.
    _source_subscription: Subscription,
    /// Toast id most recently mirrored into a component notification.
    notified_toast: Option<u64>,
    /// Set when the slider value changed without a window at hand.
    memory_sync_pending: bool,
    /// Input that should receive focus on the next frame.
    pending_focus: Option<crate::inputs::InputField>,
    /// Restore focus to the shell after a field closed.
    refocus_pending: bool,
}

/// Notification id type used to replace the previous toast in place.
struct ToastNote;

impl SealyShell {
    fn new(
        bootstrap: std::rc::Rc<sealy_core::Bootstrap>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut state = ShellState::from_bootstrap((*bootstrap).clone());
        let memory_slider = cx.new(|_| {
            SliderState::new()
                .max(32768.0)
                .min(512.0)
                .step(256.0)
                .default_value(state.effective_memory_mb() as f32)
        });
        state.memory_slider = Some(memory_slider.clone());
        let source_select = cx.new(|cx| {
            SelectState::new(
                source_options(state.language()),
                Some(IndexPath::new(source_index(
                    state.settings.download.metadata_source,
                ))),
                window,
                cx,
            )
        });
        state.source_select = Some(source_select.clone());
        let source_subscription = cx.subscribe(
            &source_select,
            |shell, _, event: &SelectEvent<SearchableVec<SharedString>>, cx| {
                if let SelectEvent::Confirm(Some(label)) = event {
                    shell.state.set_source_from_label(label);
                    cx.notify();
                }
            },
        );

        let memory_subscription =
            cx.subscribe(&memory_slider, |shell, _, event: &SliderEvent, cx| {
                let (value, commit) = match event {
                    SliderEvent::Change(value) => (value.end(), false),
                    SliderEvent::Release(value) => (value.end(), true),
                };
                shell
                    .state
                    .apply_slider_memory((value.round() as u32).max(512), commit);
                cx.notify();
            });

        let lang = state.language();
        let inputs = crate::inputs::create_map(lang, window, cx);
        state.inputs = inputs.clone();
        let mut input_subscriptions = Vec::new();
        for field in crate::inputs::InputField::ALL {
            if let Some(input) = inputs.get(&field).cloned() {
                input_subscriptions.push(cx.subscribe(
                    &input,
                    move |shell, _, event: &crate::inputs::FieldEvent, cx| {
                        shell.on_input_event(field, event, cx);
                    },
                ));
            }
        }

        // macOS sandboxing of downloaded bundles is invisible otherwise.
        let mut shell = Self {
            state,
            focus: cx.focus_handle(),
            _memory_subscription: memory_subscription,
            _input_subscriptions: input_subscriptions,
            _source_subscription: source_subscription,
            notified_toast: None,
            memory_sync_pending: false,
            pending_focus: None,
            refocus_pending: false,
        };
        if shell.state.translocated {
            shell.state.push_message(
                "macOS 正在隔离运行".to_owned(),
                "请把 SealyLauncher.app 拖到「应用程序」文件夹后重新打开；\
                 从下载目录直接双击时 macOS 会把它放进只读的临时路径，数据无法保存在旁边。"
                    .to_owned(),
                crate::state::MessageTone::Failed,
            );
        }
        if shell.state.data_fallback {
            shell.state.push_message(
                "数据目录已回退".to_owned(),
                format!(
                    "程序所在目录不可写，已改用 {}；游戏目录为 {}。",
                    shell.state.paths.data.display(),
                    shell.state.paths.game.display()
                ),
                crate::state::MessageTone::Failed,
            );
        }
        if let Some(expected) = shell.state.update_warning.take() {
            shell.state.notify(format!(
                "上次更新到 {expected} 可能未完成，请重试或手动下载安装"
            ));
        }

        // Quiet self-update check; failures never interrupt the user.
        if shell.state.settings.update.auto_check {
            shell.state.spawn_auto_update_check(cx);
        }

        // Java probing spawns processes, so it happens after the first frame.
        let game = shell.state.paths.game.clone();
        let installed_game = game.clone();
        cx.spawn(async move |this, cx| {
            let found = cx
                .background_executor()
                .spawn(async move { sealy_core::java::detect(&game) })
                .await;
            let installed = cx
                .background_executor()
                .spawn(async move { sealy_core::version::installed(&installed_game) })
                .await;
            let _ = this.update(cx, |shell, cx| {
                shell.state.java = found;
                shell.state.installed = installed;
                cx.notify();
            });
        })
        .detach();

        // Toasts dismiss themselves; a single ticker keeps every call site
        // from needing its own timer.
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(250))
                    .await;
                let updated = this.update(cx, |shell, cx| {
                    let mut dirty = shell.state.expire_due_toast();
                    // Keep the download progress bar repainting while it runs.
                    if shell.state.update_downloading {
                        dirty = true;
                    }
                    if dirty {
                        cx.notify();
                    }
                });
                if updated.is_err() {
                    return;
                }
            }
        })
        .detach();

        // The log viewer follows the file while it is open.
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(1200))
                    .await;
                let updated = this.update(cx, |shell, cx| {
                    if shell.state.logs_open {
                        shell.state.refresh_logs();
                        cx.notify();
                    }
                });
                if updated.is_err() {
                    return;
                }
            }
        })
        .detach();

        shell
    }
}

impl Render for SealyShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if window.focused(cx).is_none() {
            window.focus(&self.focus, cx);
        }

        if self.memory_sync_pending {
            self.memory_sync_pending = false;
            self.sync_memory_slider(window, cx);
        }

        if let Some(field) = self.pending_focus.take() {
            if let Some(input) = self.state.inputs.get(&field).cloned() {
                crate::inputs::focus(&input, window, cx);
            }
        } else if self.refocus_pending {
            self.refocus_pending = false;
            window.focus(&self.focus, cx);
        }

        self.sync_message_sheet(window, cx);
        self.sync_toast_notification(window, cx);
        let tokens = self.state.theme();
        let page = pages::render(&mut self.state, cx);

        div()
            .id("shell")
            .role(Role::Application)
            .aria_label("SealyLauncher")
            .track_focus(&self.focus)
            .on_action(cx.listener(|shell, _: &crate::menu::OpenHome, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::Home, cx)
            }))
            .on_action(cx.listener(|shell, _: &crate::menu::OpenInstances, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::Instances, cx)
            }))
            .on_action(cx.listener(|shell, _: &crate::menu::OpenDownloads, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::Downloads, cx)
            }))
            .on_action(cx.listener(|shell, _: &crate::menu::OpenSettings, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::Settings, cx)
            }))
            .on_action(cx.listener(|shell, _: &crate::menu::NewInstance, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::NewInstance, cx)
            }))
            .on_action(cx.listener(|shell, _: &crate::menu::ImportModpack, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::ImportModpack, cx)
            }))
            .on_action(
                cx.listener(|shell, _: &crate::menu::OpenGameFolder, _, cx| {
                    crate::menu::apply(shell, crate::menu::MenuAction::GameFolder, cx)
                }),
            )
            .on_action(
                cx.listener(|shell, _: &crate::menu::OpenDataFolder, _, cx| {
                    crate::menu::apply(shell, crate::menu::MenuAction::DataFolder, cx)
                }),
            )
            .on_action(cx.listener(|shell, _: &crate::menu::OpenLogFolder, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::LogFolder, cx)
            }))
            .on_action(cx.listener(|shell, _: &crate::menu::CheckUpdate, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::CheckUpdate, cx)
            }))
            .on_action(cx.listener(|shell, _: &crate::menu::AboutLauncher, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::About, cx)
            }))
            .on_action(
                cx.listener(|shell, _: &crate::menu::OpenRepository, _, cx| {
                    crate::menu::apply(shell, crate::menu::MenuAction::Repository, cx)
                }),
            )
            .on_action(cx.listener(|shell, _: &crate::menu::QuitLauncher, _, cx| {
                crate::menu::apply(shell, crate::menu::MenuAction::Quit, cx)
            }))
            .on_action(cx.listener(|shell, _: &InputEscape, window, cx| {
                // The editing engine handles Escape first; when it propagates,
                // the field should close and focus return to the window.
                if shell.cancel_active_input() {
                    window.focus(&shell.focus, cx);
                    cx.notify();
                }
            }))
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(tokens.bg))
            .text_color(rgb(tokens.text))
            .child(
                TitleBar::new()
                    .bg(rgb(tokens.surface))
                    .text_color(rgb(tokens.text))
                    .child(brand(&tokens))
                    .child(div().flex_1())
                    .child(message_button(&tokens, self.state.running_messages(), cx)),
            )
            .child(div().flex_1().relative().overflow_hidden().child(page))
            .on_key_down(cx.listener(|shell, event: &KeyDownEvent, _, cx| {
                shell.handle_key(&event.keystroke, cx);
            }))
    }
}

/// Localized labels for the download-source dropdown, in policy order.
fn source_options(lang: sealy_core::config::Language) -> SearchableVec<SharedString> {
    SearchableVec::new(vec![
        SharedString::from(tr(lang, "自动", "Auto")),
        SharedString::from(tr(lang, "官方", "Official")),
        SharedString::from("BMCLAPI"),
    ])
}

/// Index of a download policy in [`source_options`].
fn source_index(policy: sealy_core::config::SourcePolicy) -> usize {
    match policy {
        sealy_core::config::SourcePolicy::Auto => 0,
        sealy_core::config::SourcePolicy::Official => 1,
        sealy_core::config::SourcePolicy::Bmclapi => 2,
    }
}

/// Bell in the title bar; the badge shows how many tasks are still running.
fn message_button(
    tokens: &theme::ThemeTokens,
    running: usize,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    div()
        .id("messages-toggle")
        .role(Role::Button)
        .aria_label("messages")
        .mr(px(8.0))
        .w(px(34.0))
        .h(px(26.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .rounded(px(theme::RADIUS_TILE))
        .cursor_pointer()
        .hover({
            let bg = theme::alpha(tokens.line_soft, 0xFF);
            move |s| s.bg(rgba(bg))
        })
        .child(
            Icon::new(IconName::Bell)
                .w(px(15.0))
                .h(px(15.0))
                .text_color(rgb(tokens.muted)),
        )
        .when(running > 0, |this| {
            this.child(
                div()
                    .text_size(px(10.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(tokens.accent_text))
                    .child(running.to_string()),
            )
        })
        // Stop the title bar from arming a window drag on this click; the bar
        // starts moving the window on the next mouse move, which used to
        // swallow clicks on the bell.
        .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| {
            cx.stop_propagation()
        })
        .on_click(cx.listener(|shell, _, _, cx| {
            shell.state.toggle_messages();
            cx.notify();
        }))
        .into_any_element()
}

impl SealyShell {
    /// Applies the numeric memory editor typed in the settings page.
    fn apply_memory_draft(&mut self, cx: &mut Context<Self>) {
        let Some(text) = self.state.memory_draft.take() else {
            return;
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            cx.notify();
            return;
        }
        match trimmed.parse::<u32>() {
            Ok(value) => {
                self.state.set_max_memory(value);
                self.memory_sync_pending = true;
                cx.notify();
            }
            Err(_) => {
                let lang = self.state.language();
                self.state.notify(crate::i18n::tr(
                    lang,
                    "请输入 512-32768 之间的整数（MB）",
                    "Enter an integer between 512 and 32768 (MB)",
                ));
            }
        }
    }

    /// Pushes the current memory cap into the slider.
    pub(crate) fn sync_memory_slider(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(slider) = self.state.memory_slider.clone() {
            let value = self.state.effective_memory_mb() as f32;
            slider.update(cx, |state, cx| state.set_value(value, window, cx));
        }
    }
    /// Routes one editing event from a shared field into the page state.
    fn on_input_event(
        &mut self,
        field: crate::inputs::InputField,
        event: &crate::inputs::FieldEvent,
        cx: &mut Context<Self>,
    ) {
        use crate::inputs::FieldEvent;
        match event {
            FieldEvent::Change => {
                let text = crate::inputs::value(&self.state, field, cx);
                self.sync_field_draft(field, text);
                cx.notify();
            }
            FieldEvent::PressEnter { .. } => self.commit_field(field, cx),
            FieldEvent::Focus | FieldEvent::Blur => {}
        }
    }

    /// Keeps the draft mirrors (which commit logic reads) in step with editing.
    fn sync_field_draft(&mut self, field: crate::inputs::InputField, text: String) {
        use crate::inputs::InputField as F;
        match field {
            F::Account => self.state.account_draft = Some(text),
            F::Rename => {
                if let Some(draft) = &mut self.state.rename_draft {
                    draft.buffer = text;
                }
            }
            F::AuthlibServer => {
                if let Some(draft) = &mut self.state.authlib_draft {
                    draft.server = text;
                }
            }
            F::AuthlibUser => {
                if let Some(draft) = &mut self.state.authlib_draft {
                    draft.username = text;
                }
            }
            F::AuthlibPassword => {
                if let Some(draft) = &mut self.state.authlib_draft {
                    draft.password = text;
                }
            }
            F::JavaPath => {
                if let Some(draft) = &mut self.state.java_path_draft {
                    *draft = text;
                }
            }
            F::Proxy => {
                if let Some(draft) = &mut self.state.proxy_draft {
                    *draft = text;
                }
            }
            F::GameDir => {
                if let Some(draft) = &mut self.state.game_dir_draft {
                    *draft = text;
                }
            }
            F::Memory => {
                if let Some(draft) = &mut self.state.memory_draft {
                    *draft = text;
                }
            }
            F::LaunchWidth | F::LaunchHeight | F::LaunchMemory | F::LaunchJvm => {
                if let Some(draft) = &mut self.state.inst_launch_draft {
                    match field {
                        F::LaunchWidth => draft.width = text,
                        F::LaunchHeight => draft.height = text,
                        F::LaunchMemory => draft.max_mb = text,
                        F::LaunchJvm => draft.jvm_args = text,
                        _ => {}
                    }
                }
            }
        }
    }

    /// Commits one field (Enter in a real text input).
    fn commit_field(&mut self, field: crate::inputs::InputField, cx: &mut Context<Self>) {
        use crate::inputs::InputField as F;
        self.refocus_pending = true;
        match field {
            F::Account => self.state.commit_account(),
            F::Rename => self.state.commit_rename(),
            F::AuthlibServer | F::AuthlibUser | F::AuthlibPassword => {
                if field == F::AuthlibPassword || self.state.authlib_last_field() {
                    crate::auth_flow::start_authlib(&mut self.state, cx);
                } else {
                    // Moving to the next field keeps focus inside the form.
                    self.refocus_pending = false;
                    self.state.authlib_next_field();
                    self.pending_focus = Some(if self.state.authlib_last_field() {
                        F::AuthlibPassword
                    } else {
                        F::AuthlibUser
                    });
                }
            }
            F::JavaPath => self.commit_java_path(cx),
            F::Proxy => {
                let text = crate::inputs::value(&self.state, field, cx);
                self.state.proxy_draft = None;
                self.state.commit_proxy(text);
            }
            F::GameDir => {
                let text = crate::inputs::value(&self.state, field, cx);
                self.state.game_dir_draft = None;
                self.state.commit_game_dir(text);
            }
            F::Memory => self.apply_memory_draft(cx),
            F::LaunchWidth | F::LaunchHeight | F::LaunchMemory | F::LaunchJvm => {
                if let Err(err) = self.state.commit_instance_settings() {
                    self.state.notify(err);
                }
            }
        }
        cx.notify();
    }

    /// Probes the path typed into the "add Java" field.
    fn commit_java_path(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.state.take_java_path() else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let probed = cx
                .background_executor()
                .spawn(async move { sealy_core::java::probe(&path) })
                .await;
            let _ = this.update(cx, |shell, cx| {
                match probed {
                    Some(found) => {
                        let version = found.version.clone();
                        shell.state.add_java(found.clone());
                        shell.state.set_default_java(Some(&found.path));
                        shell
                            .state
                            .notify(format!("已添加 Java {version} 并设为默认"));
                    }
                    None => shell.state.notify("该路径不是可用的 Java 可执行文件"),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Opens one field: mirrors the initial value into its draft and focuses it.
    pub(crate) fn open_input(
        &mut self,
        field: crate::inputs::InputField,
        value: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::inputs::InputField as F;
        // Only one inline editor is open at a time.
        let _ = self.cancel_active_input();
        let value = value.into();
        match field {
            F::Account => self.state.account_draft = Some(value.clone()),
            F::Rename => {
                let old = self
                    .state
                    .instances
                    .current()
                    .map(|instance| instance.name.clone())
                    .unwrap_or_default();
                self.state.rename_draft = Some(crate::state::RenameDraft {
                    old,
                    buffer: value.clone(),
                });
            }
            F::AuthlibServer | F::AuthlibUser | F::AuthlibPassword => {
                let draft = self
                    .state
                    .authlib_draft
                    .get_or_insert_with(Default::default);
                match field {
                    F::AuthlibServer => draft.server = value.clone(),
                    F::AuthlibUser => draft.username = value.clone(),
                    _ => draft.password = value.clone(),
                }
            }
            F::JavaPath => self.state.java_path_draft = Some(value.clone()),
            F::Proxy => self.state.proxy_draft = Some(value.clone()),
            F::GameDir => self.state.game_dir_draft = Some(value.clone()),
            F::Memory => self.state.memory_draft = Some(value.clone()),
            F::LaunchWidth | F::LaunchHeight | F::LaunchMemory | F::LaunchJvm => {
                if let Some(draft) = &mut self.state.inst_launch_draft {
                    match field {
                        F::LaunchWidth => draft.width = value.clone(),
                        F::LaunchHeight => draft.height = value.clone(),
                        F::LaunchMemory => draft.max_mb = value.clone(),
                        _ => draft.jvm_args = value.clone(),
                    }
                }
            }
        }
        self.set_input_value(field, value, window, cx, true);
        cx.notify();
    }

    /// Fills the authlib form inputs and focuses the first one.
    pub(crate) fn load_authlib_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use crate::inputs::InputField as F;
        let draft = self.state.authlib_draft.clone().unwrap_or_default();
        for (field, value) in [
            (F::AuthlibServer, draft.server),
            (F::AuthlibUser, draft.username),
            (F::AuthlibPassword, draft.password),
        ] {
            self.set_input_value(field, value, window, cx, false);
        }
        self.pending_focus = Some(F::AuthlibServer);
        cx.notify();
    }

    /// Fills the instance launch-setting inputs from the draft.
    pub(crate) fn load_launch_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use crate::inputs::InputField as F;
        let Some(draft) = self.state.inst_launch_draft.clone() else {
            return;
        };
        for (field, value) in [
            (F::LaunchWidth, draft.width),
            (F::LaunchHeight, draft.height),
            (F::LaunchMemory, draft.max_mb),
            (F::LaunchJvm, draft.jvm_args),
        ] {
            self.set_input_value(field, value, window, cx, false);
        }
        self.pending_focus = Some(F::LaunchWidth);
        cx.notify();
    }

    fn set_input_value(
        &mut self,
        field: crate::inputs::InputField,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
        focus: bool,
    ) {
        let Some(input) = self.state.inputs.get(&field).cloned() else {
            return;
        };
        if focus {
            crate::inputs::focus_with_value(&input, value, window, cx);
        } else {
            input.update(cx, |state, cx| state.set_value(value, window, cx));
        }
    }

    /// Cancels whichever field is open; returns whether one was open.
    fn cancel_active_input(&mut self) -> bool {
        if self.state.memory_draft.take().is_some() {
            return true;
        }
        if self.state.proxy_draft.take().is_some() {
            return true;
        }
        if self.state.game_dir_draft.take().is_some() {
            return true;
        }
        if self.state.java_path_draft.take().is_some() {
            return true;
        }
        if self.state.account_draft.is_some() {
            self.state.cancel_account();
            return true;
        }
        if self.state.rename_draft.is_some() {
            self.state.cancel_rename();
            return true;
        }
        if self.state.authlib_draft.is_some() {
            self.state.authlib_draft = None;
            return true;
        }
        if self.state.inst_launch_draft.is_some() {
            self.state.inst_launch_draft = None;
            self.state.instance_settings_open = false;
            return true;
        }
        false
    }

    /// Mirrors the transient toast into a component notification.
    fn sync_toast_notification(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current = self.state.toast.as_ref().map(|toast| toast.id);
        if current == self.notified_toast {
            return;
        }
        let previous = self.notified_toast.take();
        self.notified_toast = current;
        let toast = self.state.toast.clone();
        let weak = cx.entity().downgrade();
        window.defer(cx, move |window, cx| {
            if let Some(id) = previous {
                window.remove_notification1::<ToastNote>(id as usize, cx);
            }
            let Some(toast) = toast else {
                return;
            };
            if weak.upgrade().is_none() {
                return;
            }
            let lang = weak
                .upgrade()
                .map(|shell| shell.update(cx, |this, _| this.state.language()))
                .unwrap_or(sealy_core::config::Language::ZhCn);
            let click_shell = weak.clone();
            let close_shell = weak.clone();
            let mut note = Notification::new()
                .message(crate::panel::soft_wrap(&toast.message))
                .id1::<ToastNote>(toast.id as usize)
                .autohide(toast.action.is_none())
                .on_click(move |_, _, cx| {
                    if let Some(shell) = click_shell.upgrade() {
                        shell.update(cx, |this, cx| {
                            this.state.dismiss_toast();
                            cx.notify();
                        });
                    }
                })
                .on_close(move |_, cx| {
                    if let Some(shell) = close_shell.upgrade() {
                        shell.update(cx, |this, cx| {
                            this.state.dismiss_toast();
                            cx.notify();
                        });
                    }
                });
            if toast.action == Some(crate::state::ToastAction::Undo) {
                let undo_shell = weak.clone();
                let label = tr(lang, "撤销", "Undo").to_owned();
                note = note.action(move |_, _, _| {
                    let target = undo_shell.clone();
                    let text = label.clone();
                    Button::new("toast-undo")
                        .small()
                        .label(text)
                        .on_click(move |_, _, cx| {
                            if let Some(shell) = target.upgrade() {
                                shell.update(cx, |this, cx| {
                                    this.state.undo();
                                    cx.notify();
                                });
                            }
                        })
                });
            }
            window.push_notification(note, cx);
        });
    }

    /// Rebuilds the download-source dropdown after a language switch.
    pub(crate) fn refresh_source_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(select) = self.state.source_select.clone() else {
            return;
        };
        let items = source_options(self.state.language());
        let index = source_index(self.state.settings.download.metadata_source);
        select.update(cx, |select, cx| {
            select.set_items(items, window, cx);
            select.set_selected_index(Some(IndexPath::new(index)), window, cx);
        });
    }

    /// Opens the component dialog that confirms deleting the selected instance.
    pub(crate) fn open_delete_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(name) = self
            .state
            .instances
            .current()
            .map(|instance| instance.name.clone())
        else {
            return;
        };
        let lang = self.state.language();
        let t = self.state.theme();
        let shell = cx.entity().downgrade();
        let entry_shell = shell.clone();
        let files_shell = shell.clone();
        window.open_dialog(cx, move |dialog, _window, _cx| {
            let title = tr(lang, "删除实例", "Delete instance");
            let label = format!("{} {name}", tr(lang, "要删除的实例：", "Instance:"));
            let hint = tr(
                lang,
                "「仅移除记录」保留游戏与存档，可在提示里撤销；「连同文件删除」会删掉整个实例目录且不可恢复。",
                "Removing the entry keeps saves and can be undone; deleting files removes the whole instance folder for good.",
            );
            let cancel = crate::components::quiet_button(tr(lang, "取消", "Cancel"), &t)
                .on_click(move |_, window, cx| window.close_dialog(cx));
            let entry_target = entry_shell.clone();
            let entry = crate::components::primary_button(tr(lang, "仅移除记录", "Remove entry"), &t)
                .on_click(move |_, window, cx| {
                    if let Some(shell) = entry_target.upgrade() {
                        shell.update(cx, |this, cx| {
                            this.state.delete_selected_instance(false);
                            cx.notify();
                        });
                    }
                    window.close_dialog(cx);
                });
            let files_target = files_shell.clone();
            let files = crate::components::quiet_button(
                tr(lang, "连同文件删除", "Delete files too"),
                &t,
            )
            .on_click(move |_, window, cx| {
                if let Some(shell) = files_target.upgrade() {
                    shell.update(cx, |this, cx| {
                        this.state.delete_selected_instance(true);
                        cx.notify();
                    });
                }
                window.close_dialog(cx);
            });
            dialog
                .title(
                    div()
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(title),
                )
                .width(px(500.0))
                .content(move |content, _, _| {
                    content
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(rgb(t.text))
                                .child(label.clone()),
                        )
                        .child(
                            div()
                                .mt(px(8.0))
                                .text_size(px(11.0))
                                .line_height(relative(1.5))
                                .text_color(rgb(t.muted))
                                .child(hint),
                        )
                })
                .footer(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .w_full()
                        .child(cancel)
                        .child(div().flex_1())
                        .child(files)
                        .child(entry),
                )
        });
    }

    /// Keeps the message-center sheet in step with `message_open`.
    ///
    /// The sheet is a component-owned overlay, so it is opened and closed
    /// imperatively after the frame that flipped the state flag.
    fn sync_message_sheet(&self, window: &mut Window, cx: &mut Context<Self>) {
        let open = self.state.message_open;
        let active = window.has_active_sheet(cx);
        if open && !active {
            let weak = cx.entity().downgrade();
            window.defer(cx, move |window, cx| {
                if weak.upgrade().is_none() {
                    return;
                }
                let build_shell = weak.clone();
                let close_shell = weak.clone();
                window.open_sheet(cx, move |sheet, _window, cx| {
                    let Some(shell) = build_shell.upgrade() else {
                        return sheet;
                    };
                    let sheet = shell.update(cx, |this, cx| {
                        crate::panel::build_sheet(sheet, &this.state, cx)
                    });
                    let close = close_shell.clone();
                    sheet.on_close(move |_, _, cx| {
                        if let Some(shell) = close.upgrade() {
                            shell.update(cx, |this, cx| {
                                this.state.message_open = false;
                                cx.notify();
                            });
                        }
                    })
                });
            });
        } else if !open && active {
            window.defer(cx, |window, cx| window.close_sheet(cx));
        }
    }

    /// Window-wide shortcuts: Enter runs the primary action, Escape steps back,
    /// digits jump between pages.
    fn handle_key(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) {
        // Escape cancels an open text field before anything else.
        if keystroke.key == "escape" && self.cancel_active_input() {
            self.refocus_pending = true;
            cx.notify();
            return;
        }

        // While a field is open its editing engine owns the keyboard; only
        // Escape (handled above) remains a window shortcut.
        if self.state.input_active() {
            return;
        }

        // Escape closes the message center before anything else.
        if keystroke.key == "escape" && self.state.message_open {
            self.state.message_open = false;
            cx.notify();
            return;
        }

        // Escape also leaves the new-instance picker.
        if keystroke.key == "escape" && self.state.new_instance_open {
            self.state.new_instance_open = false;
            cx.notify();
            return;
        }

        // Escape returns from an instance's detail page to the list.
        if keystroke.key == "escape" && self.state.route == Route::Instance {
            self.state.close_instance_detail();
            cx.notify();
            return;
        }

        // The crash analysis closes on Escape.
        if keystroke.key == "escape" && self.state.crash_open {
            self.state.close_crash_report();
            cx.notify();
            return;
        }

        // The log viewer closes on Escape.
        if keystroke.key == "escape" && self.state.logs_open {
            self.state.close_logs();
            cx.notify();
            return;
        }

        let modifiers = &keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.shift {
            return;
        }

        match keystroke.key.as_str() {
            "enter" | "space" => {
                if self.state.oobe_step.is_some() {
                    self.state.oobe_next();
                } else if self.state.route == Route::Home {
                    self.state.notify("启动流程将在 Phase 2 接入");
                } else {
                    return;
                }
                cx.notify();
            }
            "escape" if self.state.oobe_step.is_some() => {
                self.state.oobe_back();
                cx.notify();
            }
            key => {
                if self.state.oobe_step.is_none()
                    && let Some(route) = digit_route(key)
                {
                    self.state.route = route;
                    cx.notify();
                }
            }
        }
    }
}

fn digit_route(key: &str) -> Option<Route> {
    match key {
        "1" => Some(Route::Home),
        "2" => Some(Route::Instances),
        "3" => Some(Route::Downloads),
        "4" => Some(Route::Accounts),
        "5" => Some(Route::Settings),
        _ => None,
    }
}

fn brand(tokens: &theme::ThemeTokens) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(14.0))
        .child(
            img(crate::assets::ICON_PATH)
                .w(px(18.0))
                .h(px(18.0))
                .rounded(px(4.0))
                .object_fit(ObjectFit::Contain),
        )
        .child(
            div()
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(tokens.text))
                .child("SEALY"),
        )
        .child(
            div()
                .text_size(px(10.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(tokens.faint))
                .child("LAUNCHER"),
        )
        .into_any_element()
}
