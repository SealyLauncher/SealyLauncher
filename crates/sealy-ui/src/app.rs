//! Application shell: window chrome, page routing and transient notices.

use gpui_kit::component::TitleBar;

use crate::kit::*;
use crate::pages;
use crate::state::{Route, ShellState};
use crate::theme;

/// Starts the GPUI application and opens the shell window.
pub fn run(bootstrap: sealy_core::Bootstrap) {
    gpui_kit::application()
        .with_assets(crate::assets::SealyAssets)
        .run(|cx| {
            gpui_kit::init(cx);

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

            gpui_kit::open_window(options, cx, move |_, cx| {
                cx.new(|cx| SealyShell::new(bootstrap, cx))
            })
            .expect("failed to open the SealyLauncher window");
        });
}

pub(crate) struct SealyShell {
    pub(crate) state: ShellState,
    focus: FocusHandle,
}

impl SealyShell {
    fn new(bootstrap: sealy_core::Bootstrap, cx: &mut Context<Self>) -> Self {
        let shell = Self {
            state: ShellState::from_bootstrap(bootstrap),
            focus: cx.focus_handle(),
        };

        // macOS sandboxing of downloaded bundles is invisible otherwise.
        let mut shell = shell;
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
                    if shell.state.expire_due_toast() {
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

        let tokens = self.state.theme();
        let page = pages::render(&mut self.state, cx);
        let panel = self
            .state
            .message_open
            .then(|| crate::panel::render(&self.state, cx));
        let notice = self
            .state
            .toast
            .clone()
            .map(|toast| toast_element(toast, &tokens, self.state.language(), cx));

        div()
            .id("shell")
            .role(Role::Application)
            .aria_label("SealyLauncher")
            .track_focus(&self.focus)
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
            .child(
                div()
                    .flex_1()
                    .relative()
                    .overflow_hidden()
                    .child(page)
                    .children(self.state.message_open.then(|| {
                        div()
                            .id("messages-backdrop")
                            .absolute()
                            .top(px(0.0))
                            .left(px(0.0))
                            .right(px(0.0))
                            .bottom(px(0.0))
                            .on_mouse_down(
                                gpui_kit::MouseButton::Left,
                                cx.listener(|shell, _, _, cx| {
                                    shell.state.message_open = false;
                                    cx.notify();
                                }),
                            )
                    }))
                    .children(panel),
            )
            .children(notice)
            .on_key_down(cx.listener(|shell, event: &KeyDownEvent, _, cx| {
                shell.handle_key(&event.keystroke, cx);
            }))
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
    /// Window-wide shortcuts: Enter runs the primary action, Escape steps back,
    /// digits jump between pages.
    fn handle_key(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) {
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

        // The authlib form captures typing while it is open.
        if self.state.authlib_draft.is_some() {
            match keystroke.key.as_str() {
                "escape" => self.state.authlib_draft = None,
                "tab" => self.state.authlib_next_field(),
                "enter" => {
                    if self.state.authlib_last_field() {
                        crate::auth_flow::start_authlib(&mut self.state, cx);
                    } else {
                        self.state.authlib_next_field();
                    }
                }
                "backspace" => self.state.authlib_pop_char(),
                key if key.chars().count() == 1 => {
                    if let Some(ch) = key.chars().next() {
                        self.state.authlib_push_char(ch);
                    }
                }
                _ => {}
            }
            cx.notify();
            return;
        }

        // Inline name editors capture typing while they are open.
        if self.state.account_draft.is_some() || self.state.rename_draft.is_some() {
            let renaming = self.state.rename_draft.is_some();
            match keystroke.key.as_str() {
                "enter" => {
                    if renaming {
                        self.state.commit_rename();
                    } else {
                        self.state.commit_account();
                    }
                }
                "escape" => {
                    if renaming {
                        self.state.cancel_rename();
                    } else {
                        self.state.cancel_account();
                    }
                }
                "backspace" => {
                    if renaming {
                        self.state.pop_rename_char();
                    } else {
                        self.state.pop_account_char();
                    }
                }
                key if key.chars().count() == 1 => {
                    if let Some(ch) = key.chars().next() {
                        if renaming {
                            self.state.push_rename_char(ch);
                        } else {
                            self.state.push_account_char(ch);
                        }
                    }
                }
                _ => {}
            }
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

fn toast_element(
    toast: crate::state::Toast,
    tokens: &theme::ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let action = toast.action;
    let mut card = div()
        .id("notice")
        .role(Role::Button)
        .aria_label(toast.message.clone())
        .absolute()
        .bottom(px(theme::ACTION_BAR_HEIGHT + 16.0))
        .right(px(24.0))
        .max_w(px(420.0))
        .px(px(14.0))
        .py(px(10.0))
        .flex()
        .items_start()
        .gap(px(12.0))
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(tokens.block))
        .cursor_pointer()
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .text_size(px(12.0))
                .line_height(relative(1.4))
                .text_color(rgb(tokens.block_text))
                .child(crate::panel::soft_wrap(&toast.message)),
        )
        .on_click(cx.listener(|shell, _, _, cx| {
            shell.state.dismiss_toast();
            cx.notify();
        }));

    if action == Some(crate::state::ToastAction::Undo) {
        card = card.child(
            div()
                .id("notice-undo")
                .role(Role::Button)
                .aria_label("undo")
                .px(px(8.0))
                .h(px(22.0))
                .flex()
                .items_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(tokens.faint))
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(tokens.block_text))
                .cursor_pointer()
                .child(crate::i18n::tr(lang, "撤销", "Undo"))
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.state.undo();
                    cx.notify();
                })),
        );
    }

    card.into_any_element()
}
