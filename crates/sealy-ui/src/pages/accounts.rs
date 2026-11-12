//! Accounts: the offline profiles wired to the launch flow.

use sealy_core::accounts::{Account, AccountKind};
use sealy_core::config::Language;

use crate::app::SealyShell;
use crate::components::{action_bar, avatar, head_band_back, mono, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::{MessageTone, ShellState};
use crate::theme::{self, ThemeTokens};

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let rows = state
        .accounts
        .accounts
        .iter()
        .map(|account| row(account, state, &t, cx))
        .collect::<Vec<_>>();

    let draft = state.account_draft.clone();
    let login = state.microsoft_login.clone();
    let authlib = state.authlib_draft.clone();

    let body: AnyElement = if let Some(authlib) = authlib {
        authlib_form(&authlib, &t, lang, cx)
    } else if let Some(login) = login {
        login_card(&login, &t, lang, cx)
    } else if draft.is_some() {
        editor(&draft.unwrap_or_default(), &t, lang, cx)
    } else if rows.is_empty() {
        empty(&t, lang)
    } else {
        div()
            .id("account-list")
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .children(rows)
            .into_any_element()
    };

    let count = state.accounts.accounts.len();
    let count_label = format!("{count} {}", tr(lang, "个账号", "accounts"));

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band_back(tr(lang, "账号", "Accounts"), &t, cx))
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
                            "离线账号无需联网，UUID 由用户名生成",
                            "Offline accounts need no network; UUID comes from the name",
                        )))
                        .child(mono(format!("{count:02}"), &t)),
                )
                .child(div().w_full().h(px(1.0)).bg(rgb(t.line)))
                .child(body),
        )
        .child(
            action_bar(&t)
                .child(mono(count_label, &t))
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "删除", "Delete"), &t)
                        .id("account-delete")
                        .role(Role::Button)
                        .aria_label("delete-account")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            if let Some(account) = shell.state.accounts.current().cloned() {
                                let name = account.name.clone();
                                if let Some(pending) = shell.state.remove_account(&name) {
                                    shell.state.pending_undo = Some(pending);
                                    let lang = shell.state.language();
                                    let message = format!(
                                        "{} {name}",
                                        tr(lang, "已删除账号", "Deleted account")
                                    );
                                    shell
                                        .state
                                        .show_toast(message, Some(crate::state::ToastAction::Undo));
                                }
                            }
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "外置登录", "Authlib"), &t)
                        .id("account-authlib")
                        .role(Role::Button)
                        .aria_label("authlib-login")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.authlib_draft = Some(Default::default());
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "Microsoft 登录", "Sign in with Microsoft"), &t)
                        .id("account-microsoft")
                        .role(Role::Button)
                        .aria_label("microsoft-login")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            crate::auth_flow::start(&mut shell.state, cx);
                        })),
                )
                .child(
                    quiet_button(tr(lang, "添加离线账号", "Add offline"), &t)
                        .id("account-add")
                        .role(Role::Button)
                        .aria_label("add-account")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.begin_account();
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

fn empty(t: &ThemeTokens, lang: Language) -> AnyElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .child(
            Icon::new(IconName::CircleUser)
                .w(px(28.0))
                .h(px(28.0))
                .text_color(rgb(t.faint)),
        )
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "还没有账号", "No accounts yet")),
        )
        .child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
            lang,
            "点「添加账号」创建一个离线账号",
            "Press “Add account” to create an offline profile",
        )))
        .into_any_element()
}

/// Inline name editor; typing is handled by the shell's key handler.
fn editor(
    draft: &str,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let shown = if draft.is_empty() {
        tr(lang, "输入用户名", "Type a username").to_owned()
    } else {
        draft.to_owned()
    };
    let is_placeholder = draft.is_empty();

    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(14.0))
        .child(
            div()
                .w(px(420.0))
                .h(px(56.0))
                .px(px(16.0))
                .flex()
                .items_center()
                .gap(px(2.0))
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.text))
                .bg(rgb(t.surface))
                .child(
                    div()
                        .text_size(px(18.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(if is_placeholder { t.faint } else { t.text }))
                        .child(shown),
                )
                .child(div().w(px(2.0)).h(px(20.0)).bg(rgb(t.text))),
        )
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
                        .id("account-cancel")
                        .role(Role::Button)
                        .aria_label("cancel-account")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.cancel_account();
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "确认", "Confirm"), t)
                        .id("account-confirm")
                        .role(Role::Button)
                        .aria_label("confirm-account")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.commit_account();
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

/// Three-field form for authlib-injector (Yggdrasil) sign-in.
fn authlib_form(
    draft: &crate::state::AuthlibDraft,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let field = |index: usize, label: &'static str, value: String, masked: bool| {
        let focused = draft.field == index;
        let text = if masked {
            "•".repeat(value.chars().count())
        } else {
            value
        };
        div()
            .w(px(460.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(t.muted))
                    .child(label),
            )
            .child(
                div()
                    .h(px(46.0))
                    .px(px(14.0))
                    .flex()
                    .items_center()
                    .rounded(px(theme::RADIUS_TILE))
                    .border_1()
                    .border_color(rgb(if focused { t.text } else { t.line }))
                    .bg(rgb(t.surface))
                    .child(
                        div()
                            .text_size(px(15.0))
                            .text_color(rgb(t.text))
                            .child(text),
                    )
                    .child(div().w(px(2.0)).h(px(18.0)).bg(rgb(t.text))),
            )
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(16.0))
        .child(
            div()
                .text_size(px(22.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "外置登录", "Authlib sign-in")),
        )
        .child(field(
            0,
            tr(lang, "服务器地址", "Server (Yggdrasil API root)"),
            draft.server.clone(),
            false,
        ))
        .child(field(
            1,
            tr(lang, "用户名", "Username"),
            draft.username.clone(),
            false,
        ))
        .child(field(
            2,
            tr(lang, "密码", "Password"),
            draft.password.clone(),
            true,
        ))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(div().text_size(px(11.0)).text_color(rgb(t.muted)).child(tr(
                    lang,
                    "Tab 切换 · 回车登录 · Esc 取消",
                    "Tab to switch · Enter to sign in · Esc to cancel",
                )))
                .child(
                    quiet_button(tr(lang, "取消", "Cancel"), t)
                        .id("authlib-cancel")
                        .role(Role::Button)
                        .aria_label("authlib-cancel")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.authlib_draft = None;
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "登录", "Sign in"), t)
                        .id("authlib-submit")
                        .role(Role::Button)
                        .aria_label("authlib-submit")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            crate::auth_flow::start_authlib(&mut shell.state, cx);
                        })),
                ),
        )
        .into_any_element()
}

/// Device-code card shown while waiting for the browser sign-in.
fn login_card(
    login: &crate::state::MicrosoftLogin,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let code = if login.user_code.is_empty() {
        "……".to_owned()
    } else {
        login.user_code.clone()
    };
    let url = if login.verification_uri.is_empty() {
        "microsoft.com/link".to_owned()
    } else {
        login
            .verification_uri
            .trim_start_matches("https://")
            .to_owned()
    };

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
        .child(div().text_size(px(18.0)).text_color(rgb(t.text)).child(url))
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
                        .child(code),
                ),
        )
        .child(div().text_size(px(11.0)).text_color(rgb(t.muted)).child(tr(
            lang,
            "输入上方代码并完成登录",
            "Enter the code above to finish",
        )))
        .child(
            quiet_button(tr(lang, "取消登录", "Cancel"), t)
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
                            row.tone = MessageTone::Failed;
                            row.progress = None;
                            row.detail = "已取消登录".to_owned();
                        });
                    }
                    shell.state.microsoft_login = None;
                    cx.notify();
                })),
        )
        .into_any_element()
}

/// Starts the device-code flow and mirrors it into the message center.
fn row(
    account: &Account,
    state: &ShellState,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let selected = state.accounts.selected.as_deref() == Some(account.name.as_str());
    let initial = account
        .name
        .chars()
        .next()
        .map(|ch| ch.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_owned());
    let kind_label = match account.kind {
        AccountKind::Offline => tr(state.language(), "离线账号", "Offline"),
        AccountKind::Microsoft => "Microsoft",
        AccountKind::Authlib => "authlib-injector",
    };
    let short_uuid: String = account.uuid.chars().take(8).collect();
    let name = account.name.clone();

    div()
        .id(("account", account.name.len()))
        .role(Role::Button)
        .aria_label(account.name.clone())
        .h(px(72.0))
        .px(px(12.0))
        .flex()
        .items_center()
        .gap(px(14.0))
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .cursor_pointer()
        .when(selected, |this| this.bg(rgb(t.surface)))
        .hover(move |this| this.bg(rgb(t.surface)))
        .child(avatar(&initial, t))
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
                        .text_color(rgb(t.text))
                        .child(account.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(rgb(t.muted))
                        .child(format!("{kind_label} · {short_uuid}")),
                ),
        )
        .child(div().when(selected, |this| {
            this.child(mono(tr(state.language(), "当前", "CURRENT"), t))
        }))
        .on_click(cx.listener(move |shell, _, _, cx| {
            shell.state.select_account(&name);
            cx.notify();
        }))
        .into_any_element()
}
