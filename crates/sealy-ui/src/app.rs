//! Application shell: window chrome, page routing and transient notices.

use gpui_kit::component::TitleBar;

use crate::kit::*;
use crate::pages;
use crate::state::ShellState;
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
                cx.new(|_| SealyShell::new(bootstrap))
            })
            .expect("failed to open the SealyLauncher window");
        });
}

pub(crate) struct SealyShell {
    pub(crate) state: ShellState,
}

impl SealyShell {
    fn new(bootstrap: sealy_core::Bootstrap) -> Self {
        Self {
            state: ShellState::from_bootstrap(bootstrap),
        }
    }
}

impl Render for SealyShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
