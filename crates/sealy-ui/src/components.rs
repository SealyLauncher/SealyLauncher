//! Shared building blocks for the editorial shell.
//!
//! Components only own shape and colour; interaction ids and handlers stay in
//! the pages so the same tile can navigate, launch, or do nothing.

use crate::kit::*;
use crate::theme::{self, ThemeTokens};

/// Uppercase monospace label used for indices, dates and durations.
pub fn mono(text: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let text: SharedString = text.into();
    div()
        .text_size(px(10.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(t.faint))
        .child(text)
}

/// Page title band with the single large title.
pub fn head_band(title: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let title: SharedString = title.into();
    div()
        .h(px(theme::HEAD_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .px(px(theme::PAGE_PADDING))
        .bg(rgb(t.surface))
        .border_b_1()
        .border_color(rgb(t.line))
        .child(
            div()
                .text_size(px(42.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(title),
        )
}

/// Head band with a back arrow, for every page except the home screen.
pub(crate) fn head_band_back(
    title: impl Into<SharedString>,
    t: &ThemeTokens,
    cx: &mut Context<crate::app::SealyShell>,
) -> Div {
    head_band_back_to(title, crate::state::Route::Home, t, cx)
}

/// Device-code card shared by the accounts page and the first-run wizard.
///
/// Shows the code the user types on `microsoft.com/link`, a button that opens
/// the page, a copy button and the cancel action.
pub(crate) fn microsoft_login_card(
    login: &crate::state::MicrosoftLogin,
    t: &ThemeTokens,
    lang: sealy_core::config::Language,
    cx: &mut Context<crate::app::SealyShell>,
) -> AnyElement {
    use crate::i18n::tr;

    let code = if login.user_code.is_empty() {
        "……".to_owned()
    } else {
        login.user_code.clone()
    };
    let url = if login.verification_uri.is_empty() {
        "https://microsoft.com/link".to_owned()
    } else {
        login.verification_uri.clone()
    };
    let shown_url = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_owned();

    let copy_code = code.clone();
    let open_url = url.clone();

    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "在浏览器中打开", "Open in your browser")),
        )
        .child(
            div()
                .text_size(px(18.0))
                .text_color(rgb(t.text))
                .child(shown_url),
        )
        .child(
            div()
                .mt(px(6.0))
                .px(px(22.0))
                .h(px(64.0))
                .flex()
                .items_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.line))
                .bg(rgb(t.surface))
                .child(
                    div()
                        .text_size(px(28.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(t.text))
                        .child(code.clone()),
                ),
        )
        .child(div().text_size(px(11.0)).text_color(rgb(t.muted)).child(tr(
            lang,
            "输入上方代码并完成登录",
            "Enter the code above to finish",
        )))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    crate::components::quiet_button(
                        tr(lang, "打开 microsoft.com/link", "Open microsoft.com/link"),
                        t,
                    )
                    .id("microsoft-open-link")
                    .role(Role::Button)
                    .aria_label("microsoft-open-link")
                    .on_click(cx.listener(move |_, _, _, cx| {
                        let _ = sealy_core::shell::spawn_open_url_helper(&open_url);
                        cx.notify();
                    })),
                )
                .child(
                    crate::components::quiet_button(tr(lang, "复制代码", "Copy code"), t)
                        .id("microsoft-copy-code")
                        .role(Role::Button)
                        .aria_label("microsoft-copy-code")
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            cx.write_to_clipboard(crate::kit::ClipboardItem::new_string(
                                copy_code.clone(),
                            ));
                            if let Some(login) = &mut shell.state.microsoft_login {
                                login.code_copied = true;
                            }
                            let lang = shell.state.language();
                            shell
                                .state
                                .notify(tr(lang, "已复制登录代码", "Login code copied"));
                            cx.notify();
                        })),
                ),
        )
        .child(
            crate::components::quiet_button(tr(lang, "取消登录", "Cancel"), t)
                .id("microsoft-cancel")
                .role(Role::Button)
                .aria_label("microsoft-cancel")
                .on_click(cx.listener(|shell, _, _, cx| {
                    if let Some(login) = &shell.state.microsoft_login {
                        login
                            .cancel
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        let message_id = login.message_id;
                        shell.state.update_message(message_id, |row| {
                            row.tone = crate::state::MessageTone::Failed;
                            row.progress = None;
                            row.detail = "已取消登录".to_owned();
                        });
                    }
                    shell.state.microsoft_login = None;
                    cx.notify();
                })),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(rgb(t.faint))
                .child(if login.code_copied {
                    tr(
                        lang,
                        "代码已自动复制到剪贴板",
                        "Code copied to the clipboard",
                    )
                } else {
                    tr(
                        lang,
                        "代码会自动复制到剪贴板",
                        "The code is copied automatically",
                    )
                }),
        )
        .into_any_element()
}
/// Head band with a back arrow that returns to the given route.
pub(crate) fn head_band_back_to(
    title: impl Into<SharedString>,
    route: crate::state::Route,
    t: &ThemeTokens,
    cx: &mut Context<crate::app::SealyShell>,
) -> Div {
    let title: SharedString = title.into();
    let muted = t.muted;
    let line_soft = t.line_soft;

    div()
        .h(px(theme::HEAD_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(14.0))
        .px(px(theme::PAGE_PADDING))
        .bg(rgb(t.surface))
        .border_b_1()
        .border_color(rgb(t.line))
        .child(
            div()
                .id("back")
                .role(Role::Button)
                .aria_label("back")
                .w(px(34.0))
                .h(px(34.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.line_soft))
                .cursor_pointer()
                .hover(move |style| style.bg(rgb(line_soft)))
                .child(
                    Icon::new(IconName::ArrowLeft)
                        .w(px(16.0))
                        .h(px(16.0))
                        .text_color(rgb(muted)),
                )
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.state.route = route;
                    cx.notify();
                })),
        )
        .child(
            div()
                .text_size(px(42.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(title),
        )
}

/// Base tile surface.
pub fn tile(t: &ThemeTokens) -> Div {
    div()
        .relative()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.surface))
        .border_1()
        .border_color(rgb(t.line))
        .overflow_hidden()
        .cursor_pointer()
        .hover(move |s| s.border_color(rgb(t.text)))
}

/// Small square tile: icon only.
pub fn small_tile(icon: IconName, active: bool, t: &ThemeTokens) -> Div {
    let base = tile(t)
        .w(px(theme::TILE_SM))
        .h(px(theme::TILE_SM))
        .flex()
        .items_center()
        .justify_center();

    if active {
        base.bg(rgb(t.block)).border_color(rgb(t.block)).child(
            Icon::new(icon)
                .w(px(22.0))
                .h(px(22.0))
                .text_color(rgb(t.block_text)),
        )
    } else {
        base.child(
            Icon::new(icon)
                .w(px(22.0))
                .h(px(22.0))
                .text_color(rgb(t.text)),
        )
    }
}

/// Medium tile: icon above a label.
pub fn medium_tile(icon: IconName, label: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    tile(t)
        .w(px(theme::TILE_MD))
        .h(px(theme::TILE_MD))
        .flex()
        .flex_col()
        .justify_end()
        .gap(px(10.0))
        .p(px(14.0))
        .child(
            Icon::new(icon)
                .w(px(24.0))
                .h(px(24.0))
                .text_color(rgb(t.text)),
        )
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(label),
        )
}

/// Wide tile: framed icon, label, trailing arrow.
pub fn wide_tile(icon: IconName, label: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    tile(t)
        .w(px(theme::TILE_MD * 2.0 + theme::TILE_GAP))
        .h(px(theme::TILE_MD))
        .flex()
        .items_center()
        .gap(px(16.0))
        .px(px(20.0))
        .child(
            div()
                .w(px(54.0))
                .h(px(54.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.line))
                .child(
                    Icon::new(icon)
                        .w(px(26.0))
                        .h(px(26.0))
                        .text_color(rgb(t.text)),
                ),
        )
        .child(
            div()
                .text_size(px(17.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(label),
        )
        .child(div().flex_1())
        .child(
            Icon::new(IconName::ArrowRight)
                .w(px(15.0))
                .h(px(15.0))
                .text_color(rgb(t.faint)),
        )
}

/// Inverted block used for the notices tile.
pub fn block_tile(t: &ThemeTokens) -> Div {
    div()
        .w(px(theme::TILE_MD * 3.0 + theme::TILE_GAP * 2.0))
        .h(px(theme::TILE_MD * 2.0 + theme::TILE_GAP))
        .flex()
        .flex_col()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.block))
        .p(px(18.0))
        .cursor_pointer()
}

/// Column tile used for the recent-worlds list.
pub fn list_tile(t: &ThemeTokens) -> Div {
    div()
        .w(px(theme::TILE_MD * 3.0 + theme::TILE_GAP * 2.0))
        .h(px(theme::TILE_MD * 2.0 + theme::TILE_GAP))
        .flex()
        .flex_col()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.surface))
        .border_1()
        .border_color(rgb(t.line))
        .p(px(18.0))
}

/// Persistent bottom action bar.
pub fn action_bar(t: &ThemeTokens) -> Div {
    div()
        .h(px(theme::ACTION_BAR_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(16.0))
        .px(px(24.0))
        .bg(rgb(t.surface))
        .border_t_1()
        .border_color(rgb(t.line))
}

/// Filled primary action.
pub fn primary_button(label: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    div()
        .h(px(58.0))
        .px(px(26.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.accent))
        .cursor_pointer()
        .child(
            div()
                .text_size(px(20.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.accent_text))
                .child(label),
        )
}

/// Quiet bordered button.
pub fn quiet_button(label: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    div()
        .h(px(40.0))
        .px(px(16.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .border_1()
        .border_color(rgb(t.line))
        .cursor_pointer()
        .hover(move |s| s.border_color(rgb(t.text)))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(t.text))
                .child(label),
        )
}

/// Bordered button that reports a current selection.
pub fn choice_button(label: impl Into<SharedString>, selected: bool, t: &ThemeTokens) -> Div {
    let label: SharedString = label.into();
    let (bg, fg, border) = if selected {
        (t.block, t.block_text, t.block)
    } else {
        (t.surface, t.text, t.line)
    };

    div()
        .h(px(40.0))
        .px(px(16.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(bg))
        .border_1()
        .border_color(rgb(border))
        .cursor_pointer()
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(fg))
                .child(label),
        )
}

/// Square icon button.
pub fn icon_button(icon: IconName, t: &ThemeTokens) -> Div {
    div()
        .w(px(38.0))
        .h(px(38.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .border_1()
        .border_color(rgb(t.line))
        .cursor_pointer()
        .hover(move |s| s.border_color(rgb(t.text)))
        .child(
            Icon::new(icon)
                .w(px(17.0))
                .h(px(17.0))
                .text_color(rgb(t.muted)),
        )
}

/// Square avatar placeholder.
pub fn avatar(letter: impl Into<SharedString>, t: &ThemeTokens) -> Div {
    let letter: SharedString = letter.into();
    div()
        .w(px(42.0))
        .h(px(42.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.block))
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(t.block_text))
                .child(letter),
        )
}

/// One row of a settings list.
pub fn field_row(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
    t: &ThemeTokens,
) -> Div {
    let label: SharedString = label.into();
    let value: SharedString = value.into();
    div()
        .h(px(48.0))
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
        .child(
            div()
                .text_size(px(13.0))
                .text_color(rgb(t.muted))
                .child(value),
        )
}
