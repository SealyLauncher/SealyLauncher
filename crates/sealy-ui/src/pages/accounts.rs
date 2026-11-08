//! Accounts: the offline profiles wired to the launch flow.

use sealy_core::accounts::{Account, AccountKind};
use sealy_core::config::Language;

use crate::app::SealyShell;
use crate::components::{action_bar, avatar, head_band_back, mono, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::ShellState;
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

    let body: AnyElement = if draft.is_some() {
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
        .child(head_band_back("04", tr(lang, "账号", "Accounts"), &t, cx))
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
                                shell.state.remove_account(&account.name);
                            }
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "添加账号", "Add account"), &t)
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
