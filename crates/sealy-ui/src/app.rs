//! Application shell: window chrome, page routing and transient notices.

use gpui_kit::component::TitleBar;

use crate::kit::*;
use crate::pages;
use crate::state::{Route, ShellState};
use crate::theme;

/// Starts the GPUI application and opens the shell window.
pub fn run(bootstrap: sealy_core::Bootstrap) {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
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
        Self {
            state: ShellState::from_bootstrap(bootstrap),
            focus: cx.focus_handle(),
        }
    }
}

impl Render for SealyShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if window.focused(cx).is_none() {
            window.focus(&self.focus, cx);
        }

        let tokens = self.state.theme();
        let page = pages::render(&self.state, cx);
        let notice = self
            .state
            .notice
            .clone()
            .map(|message| toast(message, &tokens, cx));

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
                    .child(brand(&tokens)),
            )
            .child(div().flex_1().overflow_hidden().child(page))
            .children(notice)
            .on_key_down(cx.listener(|shell, event: &KeyDownEvent, _, cx| {
                shell.handle_key(&event.keystroke, cx);
            }))
    }
}

impl SealyShell {
    /// Window-wide shortcuts: Enter runs the primary action, Escape steps back,
    /// digits jump between pages.
    fn handle_key(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) {
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
        .gap(px(10.0))
        .px(px(14.0))
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

fn toast(message: String, tokens: &theme::ThemeTokens, cx: &mut Context<SealyShell>) -> AnyElement {
    div()
        .id("notice")
        .role(Role::Button)
        .aria_label(message.clone())
        .absolute()
        .bottom(px(theme::ACTION_BAR_HEIGHT + 16.0))
        .right(px(24.0))
        .px(px(14.0))
        .py(px(10.0))
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(tokens.block))
        .cursor_pointer()
        .child(
            div()
                .text_size(px(12.0))
                .text_color(rgb(tokens.block_text))
                .child(message),
        )
        .on_click(cx.listener(|shell, _, _, cx| {
            shell.state.notice = None;
            cx.notify();
        }))
        .into_any_element()
}
