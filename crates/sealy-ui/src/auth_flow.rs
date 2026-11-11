//! Shared Microsoft sign-in flow, used by the accounts page and the wizard.

use crate::app::SealyShell;
use crate::i18n::tr;
use crate::kit::Context;
use crate::state::{MessageTone, ShellState};

pub(crate) fn start(state: &mut ShellState, cx: &mut Context<SealyShell>) {
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
    {
        let handle = cancel.clone();
        state.update_message(message_id, move |row| row.cancel = Some(handle));
    }
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
                        row.cancel = None;
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
                let _ = this.update(cx, |shell, cx| {
                    shell.state.microsoft_login = None;
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Failed;
                        row.cancel = None;
                        row.progress = None;
                        row.cancel = None;
                        row.detail = "已取消".to_owned();
                    });
                    cx.notify();
                });
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
                            row.cancel = None;
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
                let client_id_clone = client_id.clone();
                async move {
                    let token = sealy_core::microsoft::MsaToken {
                        access_token: msa_access,
                        refresh_token: None,
                    };
                    sealy_core::microsoft::complete_login(&token, &client_id_clone)
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
                        server: None,
                    });
                    shell.state.tokens.store(
                        &uuid,
                        sealy_core::auth::StoredTokens {
                            refresh_token: refresh,
                            minecraft_token: Some(access),
                            expires_at,
                            ..Default::default()
                        },
                    );
                    shell.state.save_accounts();
                    shell.state.save_tokens();
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Done;
                        row.cancel = None;
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
                        row.cancel = None;
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

/// Signs in through an authlib-injector server (Yggdrasil).
pub(crate) fn start_authlib(state: &mut ShellState, cx: &mut Context<SealyShell>) {
    let Some(draft) = state.authlib_draft.clone() else {
        return;
    };

    if draft.server.trim().is_empty()
        || draft.username.trim().is_empty()
        || draft.password.is_empty()
    {
        return;
    }

    let lang = state.language();
    let message_id = state.push_message(
        "外置登录".to_owned(),
        tr(lang, "正在向服务器验证…", "Contacting the server…").to_owned(),
        MessageTone::Running,
    );
    state.message_open = true;

    let server = sealy_core::authlib::normalize_api(&draft.server);
    let username = draft.username.clone();
    let password = draft.password.clone();

    cx.spawn(async move |this, cx| {
        let login = {
            let server = server.clone();
            cx.background_executor()
                .spawn(async move { sealy_core::authlib::login(&server, &username, &password) })
                .await
        };

        match login {
            Ok(session) => {
                let name = session.name.clone();
                let uuid = session.uuid.clone();
                let _ = this.update(cx, |shell, cx| {
                    shell.state.authlib_draft = None;
                    let account = sealy_core::accounts::Account {
                        name: name.clone(),
                        uuid: uuid.clone(),
                        kind: sealy_core::accounts::AccountKind::Authlib,
                        server: Some(server.clone()),
                    };
                    shell.state.accounts.add(account);
                    let account = shell.state.accounts.current().cloned();
                    if let Some(account) = account {
                        sealy_core::auth::cached_authlib(
                            &mut shell.state.tokens,
                            &account,
                            &session,
                        );
                    }
                    shell.state.save_accounts();
                    shell.state.save_tokens();
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Done;
                        row.progress = Some(1.0);
                        row.cancel = None;
                        row.detail = format!("已登录 {name}");
                    });
                    cx.notify();
                });
            }
            Err(err) => {
                let detail = err.to_string();
                let _ = this.update(cx, |shell, cx| {
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Failed;
                        row.progress = None;
                        row.cancel = None;
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
pub(crate) fn dashed_uuid(raw: &str) -> String {
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
