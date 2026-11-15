//! Single-line text fields backed by GPUI's editing engine.
//!
//! The launcher used to draw its own "inputs" with a hand-rolled key handler,
//! which meant no caret, no selection, no click-to-place-cursor and no
//! horizontal scrolling. Every field is now a real [`InputState`] entity from
//! `gpui-component`, rendered through [`Input`] with the paper theme.

use std::collections::BTreeMap;

use gpui_kit::component::input::{Input, InputState};
use sealy_core::config::Language;

use crate::app::SealyShell;
use crate::kit::*;
use crate::state::ShellState;
use crate::theme::{self, ThemeTokens};

/// Every editable text field the shell owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputField {
    /// Offline account name (accounts page and first-run wizard).
    Account,
    /// Instance rename buffer.
    Rename,
    /// Authlib server URL.
    AuthlibServer,
    /// Authlib username.
    AuthlibUser,
    /// Authlib password.
    AuthlibPassword,
    /// Java executable path for "add Java".
    JavaPath,
    /// HTTP proxy (`host:port`).
    Proxy,
    /// Custom game directory.
    GameDir,
    /// Manual heap cap in MiB.
    Memory,
    /// Instance launch window width.
    LaunchWidth,
    /// Instance launch window height.
    LaunchHeight,
    /// Instance launch heap cap.
    LaunchMemory,
    /// Instance launch extra JVM arguments.
    LaunchJvm,
}

impl InputField {
    /// Every field, in creation order.
    pub const ALL: [Self; 13] = [
        Self::Account,
        Self::Rename,
        Self::AuthlibServer,
        Self::AuthlibUser,
        Self::AuthlibPassword,
        Self::JavaPath,
        Self::Proxy,
        Self::GameDir,
        Self::Memory,
        Self::LaunchWidth,
        Self::LaunchHeight,
        Self::LaunchMemory,
        Self::LaunchJvm,
    ];

    /// Placeholder shown while the field is empty.
    pub fn placeholder(self, lang: Language) -> &'static str {
        use crate::i18n::tr;
        match self {
            Self::Account => tr(lang, "输入用户名", "Type a username"),
            Self::Rename => tr(lang, "输入新名称", "New name"),
            Self::AuthlibServer => tr(
                lang,
                "https://example.com/api/yggdrasil",
                "https://example.com/api/yggdrasil",
            ),
            Self::AuthlibUser => tr(lang, "用户名", "Username"),
            Self::AuthlibPassword => tr(lang, "密码", "Password"),
            Self::JavaPath => tr(lang, "/path/to/java", "/path/to/java"),
            Self::Proxy => "host:port",
            Self::GameDir => tr(lang, "游戏目录路径", "Game folder path"),
            Self::Memory => tr(lang, "最大内存（MB）", "Max memory (MB)"),
            Self::LaunchWidth => tr(
                lang,
                "窗口宽度，留空为默认",
                "Window width, empty for default",
            ),
            Self::LaunchHeight => tr(
                lang,
                "窗口高度，留空为默认",
                "Window height, empty for default",
            ),
            Self::LaunchMemory => tr(
                lang,
                "内存（MB），留空为默认",
                "Memory (MB), empty for default",
            ),
            Self::LaunchJvm => tr(lang, "额外 JVM 参数", "Extra JVM arguments"),
        }
    }

    /// Accessibility label for UI automation and screen readers.
    pub fn aria(self) -> &'static str {
        match self {
            Self::Account => "input-account",
            Self::Rename => "input-rename",
            Self::AuthlibServer => "input-authlib-server",
            Self::AuthlibUser => "input-authlib-user",
            Self::AuthlibPassword => "input-authlib-password",
            Self::JavaPath => "input-java-path",
            Self::Proxy => "input-proxy",
            Self::GameDir => "input-game-dir",
            Self::Memory => "input-memory",
            Self::LaunchWidth => "input-launch-width",
            Self::LaunchHeight => "input-launch-height",
            Self::LaunchMemory => "input-launch-memory",
            Self::LaunchJvm => "input-launch-jvm",
        }
    }

    /// Whether the text must be masked (password fields).
    pub fn masked(self) -> bool {
        self == Self::AuthlibPassword
    }
}

/// Input entities keyed by field.
pub type InputMap = BTreeMap<InputField, Entity<InputState>>;

/// Creates one editing entity per field with the current placeholder text.
pub(crate) fn create_map(
    lang: Language,
    window: &mut Window,
    cx: &mut Context<SealyShell>,
) -> InputMap {
    let mut map = InputMap::new();
    for field in InputField::ALL {
        let placeholder = field.placeholder(lang).to_owned();
        let masked = field.masked();
        let entity = cx.new(|cx| {
            let state = InputState::new(&mut *window, cx).placeholder(placeholder);
            if masked { state.masked(true) } else { state }
        });
        map.insert(field, entity);
    }
    map
}

/// Updates every placeholder after a language switch.
pub(crate) fn relocalize(
    inputs: &InputMap,
    lang: Language,
    window: &mut Window,
    cx: &mut Context<SealyShell>,
) {
    for (field, input) in inputs {
        let placeholder = field.placeholder(lang).to_owned();
        input.update(cx, |state, cx| {
            state.set_placeholder(placeholder, window, cx);
        });
    }
}

/// Renders one field; the field must exist in the state map.
pub(crate) fn field(
    state: &ShellState,
    field: InputField,
    width: f32,
    t: &ThemeTokens,
) -> AnyElement {
    let Some(input) = state.inputs.get(&field) else {
        return div().into_any_element();
    };
    Input::new(input)
        .aria_label(field.aria())
        .w(px(width))
        .px(px(10.0))
        .rounded(px(theme::RADIUS_TILE))
        .border_color(rgb(t.line))
        .bg(rgb(t.surface))
        .text_size(px(12.0))
        .text_color(rgb(t.text))
        .into_any_element()
}

/// Focuses a field and replaces its content, then selects nothing.
pub(crate) fn focus_with_value(
    input: &Entity<InputState>,
    value: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut Context<SealyShell>,
) {
    let value = value.into();
    input.update(cx, |state, cx| {
        state.set_value(value, window, cx);
        state.focus(window, cx);
    });
}

/// Focuses a field without touching its content.
pub(crate) fn focus(input: &Entity<InputState>, window: &mut Window, cx: &mut Context<SealyShell>) {
    input.update(cx, |state, cx| state.focus(window, cx));
}

/// Current text of one field.
pub(crate) fn value(state: &ShellState, field: InputField, cx: &App) -> String {
    state
        .inputs
        .get(&field)
        .map(|input| input.read(cx).value().to_string())
        .unwrap_or_default()
}

/// Event type re-exported for the shell subscription.
pub use gpui_kit::component::input::InputEvent as FieldEvent;
