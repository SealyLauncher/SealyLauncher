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

    let body: AnyElement = if let Some(login) = login {
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
                    quiet_button(tr(lang, "Microsoft 登录", "Sign in with Microsoft"), &t)
                        .id("account-microsoft")
                        .role(Role::Button)
                        .aria_label("microsoft-login")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            start_microsoft(&mut shell.state, cx);
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
fn start_microsoft(state: &mut ShellState, cx: &mut Context<SealyShell>) {
    if state.microsoft_login.is_some() {
        return;
    }

    let lang = state.language();
    let client_id = state.settings.auth.microsoft_client_id.clone();
    let message_id = state.push_message(
        "Microsoft 登录".to_owned(),
        tr(lang, "正在申请设备码…", "Requesting a device code…").to_owned(),
        MessageTone::Running,
    );
    state.message_open = true;

    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    state.microsoft_login = Some(crate::state::MicrosoftLogin {
        message_id,
        user_code: String::new(),
        verification_uri: String::new(),
        cancel: cancel.clone(),
    });

    cx.spawn(async move |this, cx| {
        let request = cx
            .background_executor()
            .spawn({
                let client_id = client_id.clone();
                async move { sealy_core::microsoft::request_device_code(&client_id) }
            })
            .await;

        let code = match request {
            Ok(code) => code,
            Err(err) => {
                let detail = err.to_string();
                let _ = this.update(cx, |shell, cx| {
                    shell.state.microsoft_login = None;
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Failed;
                        row.progress = None;
                        row.detail = detail;
                    });
                    cx.notify();
                });
                return;
            }
        };

        {
            let user_code = code.user_code.clone();
            let verification_uri = code.verification_uri.clone();
            let detail = format!("{verification_uri} · {user_code}");
            let _ = this.update(cx, |shell, cx| {
                if let Some(login) = &mut shell.state.microsoft_login {
                    login.user_code = user_code;
                    login.verification_uri = verification_uri;
                }
                shell.state.update_message(message_id, |row| {
                    row.detail = detail;
                });
                cx.notify();
            });
        }

        let msa = loop {
            let interval = code.interval.max(1);
            cx.background_executor()
                .timer(std::time::Duration::from_secs(interval))
                .await;

            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            let poll = cx
                .background_executor()
                .spawn({
                    let client_id = client_id.clone();
                    let device = code.clone();
                    async move { sealy_core::microsoft::poll_device_code(&client_id, &device) }
                })
                .await;

            match poll {
                Ok(sealy_core::microsoft::Poll::Pending) => {}
                Ok(sealy_core::microsoft::Poll::SlowDown) => {
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(5))
                        .await;
                }
                Ok(sealy_core::microsoft::Poll::Ready(token)) => break token,
                Err(err) => {
                    let detail = err.to_string();
                    let _ = this.update(cx, |shell, cx| {
                        shell.state.microsoft_login = None;
                        shell.state.update_message(message_id, |row| {
                            row.tone = MessageTone::Failed;
                            row.progress = None;
                            row.detail = detail;
                        });
                        cx.notify();
                    });
                    return;
                }
            }
        };

        let session = cx
            .background_executor()
            .spawn({
                let msa_access = msa.access_token.clone();
                async move {
                    let token = sealy_core::microsoft::MsaToken {
                        access_token: msa_access,
                        refresh_token: None,
                    };
                    sealy_core::microsoft::complete_login(&token)
                }
            })
            .await;

        match session {
            Ok((profile, mc)) => {
                let uuid = dashed_uuid(&profile.id);
                let name = profile.name.clone();
                let refresh = msa.refresh_token.clone();
                let expires_at = sealy_core::auth::now_epoch() + mc.expires_in;
                let access = mc.access_token.clone();

                let _ = this.update(cx, |shell, cx| {
                    shell.state.microsoft_login = None;
                    shell.state.accounts.add(sealy_core::accounts::Account {
                        name: name.clone(),
                        uuid: uuid.clone(),
                        kind: sealy_core::accounts::AccountKind::Microsoft,
                    });
                    shell.state.tokens.store(
                        &uuid,
                        sealy_core::auth::StoredTokens {
                            refresh_token: refresh,
                            minecraft_token: Some(access),
                            expires_at,
                        },
                    );
                    shell.state.save_accounts();
                    shell.state.save_tokens();
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Done;
                        row.progress = Some(1.0);
                        row.detail = format!("已登录 {name}");
                    });
                    cx.notify();
                });
            }
            Err(err) => {
                let detail = err.to_string();
                let _ = this.update(cx, |shell, cx| {
                    shell.state.microsoft_login = None;
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Failed;
                        row.progress = None;
                        row.detail = detail;
                    });
                    cx.notify();
                });
            }
        }
    })
    .detach();
}

/// Turns the undashed UUID from Minecraft services into the dashed form.
fn dashed_uuid(raw: &str) -> String {
    if raw.len() == 32 {
        format!(
            "{}-{}-{}-{}-{}",
            &raw[0..8],
            &raw[8..12],
            &raw[12..16],
            &raw[16..20],
            &raw[20..32]
        )
    } else {
        raw.to_owned()
    }
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
