//! First-run wizard: language, audio, done.
//!
//! Every step keeps the same three zones as the rest of the launcher: a
//! breadcrumb header, one centred decision, and the persistent action bar.

use sealy_core::config::Language;

use crate::app::SealyShell;
use crate::components::{action_bar, choice_button, primary_button, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::{OOBE_STEPS, ShellState};
use crate::theme::{self, ThemeTokens};

pub(crate) fn render(state: &ShellState, step: usize, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let (crumb, sub, message) = match step {
        0 => (
            tr(lang, "语言", "Language"),
            tr(lang, "\\ 显示语言", "\\ Display language"),
            tr(
                lang,
                "选择您阅读界面时使用的语言",
                "Choose the language for the interface",
            ),
        ),
        1 => (
            tr(lang, "个性化", "Personalize"),
            tr(lang, "\\ 音乐与音效", "\\ Music and sound"),
            tr(
                lang,
                "随时可以在设置中修改音频选项",
                "You can change this later in Settings",
            ),
        ),
        2 => (
            tr(lang, "账号", "Account"),
            tr(lang, "\\ 玩家身份", "\\ Player profile"),
            tr(
                lang,
                "启动器需要一个账号才能继续",
                "The launcher needs one account to continue",
            ),
        ),
        _ => (
            tr(lang, "设置向导", "Setup"),
            tr(lang, "\\ 已完成", "\\ Done"),
            "",
        ),
    };

    let body = match step {
        0 => step_language(state, &t, cx),
        1 => step_audio(state, &t, cx),
        2 => step_account(state, &t, cx),
        _ => step_done(state, &t, cx),
    };

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(
            div()
                .flex_shrink_0()
                .px(px(48.0))
                .pt(px(36.0))
                .flex()
                .items_baseline()
                .gap(px(12.0))
                .child(
                    div()
                        .text_size(px(30.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(crumb),
                )
                .child(
                    div()
                        .text_size(px(26.0))
                        .text_color(rgb(t.muted))
                        .child(sub),
                ),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(body),
        )
        .child(action_bar(&t).child(actions(state, step, message, &t, cx)))
        .into_any_element()
}

fn step_language(state: &ShellState, t: &ThemeTokens, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(22.0))
        .child(glyph(IconName::Globe, t))
        .child(
            div()
                .text_size(px(22.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "您好", "Hello")),
        )
        .child(
            div()
                .flex()
                .gap(px(10.0))
                .child(
                    choice_button("简体中文", lang == Language::ZhCn, t)
                        .id("oobe-lang-zh")
                        .role(Role::Button)
                        .aria_label("简体中文")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_language(Language::ZhCn);
                            cx.notify();
                        })),
                )
                .child(
                    choice_button("English", lang == Language::EnUs, t)
                        .id("oobe-lang-en")
                        .role(Role::Button)
                        .aria_label("English")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.set_language(Language::EnUs);
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

fn step_audio(state: &ShellState, t: &ThemeTokens, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();
    let on = state.settings.appearance.background_music;

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(22.0))
        .child(glyph(IconName::Volume2, t))
        .child(
            div()
                .text_size(px(22.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "是否播放背景音乐", "Play background music?")),
        )
        .child(
            div()
                .flex()
                .gap(px(14.0))
                .child(
                    radio_card(
                        tr(lang, "关", "Off"),
                        tr(
                            lang,
                            "不播放任何声音与音效",
                            "No music and no interface sounds",
                        ),
                        !on,
                        t,
                    )
                    .id("oobe-audio-off")
                    .role(Role::Button)
                    .aria_label(tr(lang, "关", "Off"))
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.settings.appearance.background_music = false;
                        cx.notify();
                    })),
                )
                .child(
                    radio_card(
                        tr(lang, "开", "On"),
                        tr(
                            lang,
                            "根据当前主题播放背景音乐与音效",
                            "Music and sounds follow the current theme",
                        ),
                        on,
                        t,
                    )
                    .id("oobe-audio-on")
                    .role(Role::Button)
                    .aria_label(tr(lang, "开", "On"))
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.settings.appearance.background_music = true;
                        cx.notify();
                    })),
                ),
        )
        .into_any_element()
}

fn step_account(state: &ShellState, t: &ThemeTokens, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();

    if let Some(login) = &state.microsoft_login {
        return step_account_code(login, t, lang, cx);
    }

    if let Some(draft) = &state.account_draft {
        return offline_editor(draft, t, lang, cx);
    }

    if let Some(account) = state.accounts.current() {
        return div()
            .w(px(560.0))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(16.0))
            .child(
                div()
                    .text_size(px(26.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(t.text))
                    .child(tr(lang, "账号已就绪", "Account ready")),
            )
            .child(
                div()
                    .w_full()
                    .h(px(64.0))
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .rounded(px(theme::RADIUS_TILE))
                    .border_1()
                    .border_color(rgb(t.line))
                    .bg(rgb(t.surface))
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(t.text))
                            .child(account.name.clone()),
                    )
                    .child(div().text_size(px(11.0)).text_color(rgb(t.muted)).child(
                        match account.kind {
                            sealy_core::accounts::AccountKind::Microsoft => {
                                tr(lang, "正版账号", "Microsoft account")
                            }
                            sealy_core::accounts::AccountKind::Authlib => {
                                tr(lang, "外置登录", "Authlib account")
                            }
                            sealy_core::accounts::AccountKind::Offline => {
                                tr(lang, "离线账号", "Offline account")
                            }
                        },
                    )),
            )
            .into_any_element();
    }

    div()
        .w(px(660.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(20.0))
        .child(
            div()
                .text_size(px(24.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "先添加一个账号", "Add an account first")),
        )
        .child(
            div()
                .flex()
                .gap(px(14.0))
                .child(
                    account_card(
                        tr(lang, "离线账号", "Offline account"),
                        tr(
                            lang,
                            "不需要联网，输入名字即可开始",
                            "No network needed — just pick a name",
                        ),
                        t,
                    )
                    .id("oobe-account-offline")
                    .role(Role::Button)
                    .aria_label("oobe-offline")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.account_draft = Some(String::new());
                        cx.notify();
                    })),
                )
                .child(
                    account_card(
                        tr(lang, "Microsoft 登录", "Microsoft sign-in"),
                        tr(
                            lang,
                            "用正版账号登录，自动获取皮肤",
                            "Sign in with a Microsoft account",
                        ),
                        t,
                    )
                    .id("oobe-account-microsoft")
                    .role(Role::Button)
                    .aria_label("oobe-microsoft")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        crate::auth_flow::start(&mut shell.state, cx);
                        cx.notify();
                    })),
                ),
        )
        .into_any_element()
}

/// Device-code card shown while the wizard waits for the browser sign-in.
fn step_account_code(
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
        .flex()
        .flex_col()
        .items_center()
        .gap(px(14.0))
        .child(
            div()
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "在浏览器中打开", "Open in your browser")),
        )
        .child(div().text_size(px(20.0)).text_color(rgb(t.text)).child(url))
        .child(
            div()
                .mt(px(4.0))
                .px(px(26.0))
                .h(px(72.0))
                .flex()
                .items_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.line))
                .bg(rgb(t.surface))
                .child(
                    div()
                        .text_size(px(32.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(t.text))
                        .child(code),
                ),
        )
        .child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
            lang,
            "输入上方代码并完成登录，完成后自动继续",
            "Enter the code above; the wizard continues automatically",
        )))
        .child(
            quiet_button(tr(lang, "取消登录", "Cancel"), t)
                .id("oobe-microsoft-cancel")
                .role(Role::Button)
                .aria_label("oobe-microsoft-cancel")
                .on_click(cx.listener(|shell, _, _, cx| {
                    if let Some(login) = &shell.state.microsoft_login {
                        login
                            .cancel
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                    shell.state.microsoft_login = None;
                    cx.notify();
                })),
        )
        .into_any_element()
}

/// Inline name editor used for the offline account.
fn offline_editor(
    draft: &str,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
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
                .child(tr(lang, "输入玩家名字", "Pick a player name")),
        )
        .child(
            div()
                .w(px(380.0))
                .h(px(56.0))
                .px(px(16.0))
                .flex()
                .items_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.text))
                .bg(rgb(t.surface))
                .child(
                    div()
                        .text_size(px(18.0))
                        .text_color(rgb(t.text))
                        .child(draft.to_owned()),
                )
                .child(div().w(px(2.0)).h(px(20.0)).bg(rgb(t.text))),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(div().text_size(px(11.0)).text_color(rgb(t.muted)).child(tr(
                    lang,
                    "回车确认 · Esc 取消",
                    "Enter to confirm · Esc to cancel",
                )))
                .child(
                    quiet_button(tr(lang, "取消", "Cancel"), t)
                        .id("oobe-account-cancel")
                        .role(Role::Button)
                        .aria_label("oobe-account-cancel")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.cancel_account();
                            cx.notify();
                        })),
                )
                .child(
                    quiet_button(tr(lang, "确认", "Confirm"), t)
                        .id("oobe-account-confirm")
                        .role(Role::Button)
                        .aria_label("oobe-account-confirm")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.commit_account();
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

fn account_card(title: &str, description: &str, t: &ThemeTokens) -> Div {
    div()
        .w(px(280.0))
        .p(px(20.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.surface))
        .border_1()
        .border_color(rgb(t.line))
        .cursor_pointer()
        .child(
            div()
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(title.to_owned()),
        )
        .child(
            div()
                .text_size(px(12.0))
                .line_height(relative(1.5))
                .text_color(rgb(t.muted))
                .child(description.to_owned()),
        )
}

fn step_done(state: &ShellState, t: &ThemeTokens, cx: &mut Context<SealyShell>) -> AnyElement {
    let lang = state.language();

    div()
        .w(px(640.0))
        .flex()
        .flex_col()
        .items_start()
        .gap(px(14.0))
        .child(
            div()
                .text_size(px(34.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(tr(lang, "可以开始了。", "You are all set.")),
        )
        .child(
            div()
                .text_size(px(14.0))
                .line_height(relative(1.6))
                .text_color(rgb(t.muted))
                .child(tr(
                    lang,
                    "左上角的菜单按钮或键盘上的 HOME 键可以随时回到主界面。",
                    "The menu button in the title bar, or the HOME key, returns you here at any time.",
                )),
        )
        .child(
            div()
                .id("oobe-install")
                .mt(px(6.0))
                .flex()
                .items_center()
                .gap(px(8.0))
                .cursor_pointer()
                .border_b_1()
                .border_color(rgb(t.text))
                .pb(px(2.0))
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(t.text))
                        .child(tr(
                            lang,
                            "需要继续安装 Minecraft 核心吗？",
                            "Install a Minecraft core now?",
                        )),
                )
                .child(
                    Icon::new(IconName::ArrowRight)
                        .w(px(14.0))
                        .h(px(14.0))
                        .text_color(rgb(t.text)),
                )
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.state.oobe_step = None;
                    shell.state.route = crate::state::Route::Downloads;
                    cx.notify();
                })),
        )
        .into_any_element()
}

fn glyph(icon: IconName, t: &ThemeTokens) -> Div {
    div()
        .w(px(84.0))
        .h(px(84.0))
        .flex()
        .items_center()
        .justify_center()
        .child(
            Icon::new(icon)
                .w(px(66.0))
                .h(px(66.0))
                .text_color(rgb(t.text)),
        )
}

fn radio_card(
    name: &'static str,
    description: &'static str,
    selected: bool,
    t: &ThemeTokens,
) -> Div {
    let border = if selected { t.text } else { t.line };

    div()
        .w(px(300.0))
        .p(px(20.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.surface))
        .border_1()
        .border_color(rgb(border))
        .cursor_pointer()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .w(px(16.0))
                        .h(px(16.0))
                        .rounded_full()
                        .border_1()
                        .border_color(rgb(border))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .w(px(8.0))
                                .h(px(8.0))
                                .rounded_full()
                                .bg(rgb(if selected { t.text } else { t.surface })),
                        ),
                )
                .child(
                    div()
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(name),
                ),
        )
        .child(
            div()
                .text_size(px(12.0))
                .line_height(relative(1.5))
                .text_color(rgb(t.muted))
                .child(description),
        )
}

fn actions(
    state: &ShellState,
    step: usize,
    message: &'static str,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let lang = state.language();
    let last = step + 1 >= OOBE_STEPS;

    let mut bar = div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(16.0))
        .child(
            div()
                .text_size(px(13.0))
                .text_color(rgb(t.muted))
                .child(message),
        )
        .child(div().flex_1());

    if state.oobe_step.unwrap_or(0) > 0 {
        bar = bar.child(
            quiet_button(tr(lang, "上一步", "Back"), t)
                .h(px(58.0))
                .id("oobe-back")
                .role(Role::Button)
                .aria_label(tr(lang, "上一步", "Back"))
                .on_click(cx.listener(|shell, _, _, cx| {
                    shell.state.oobe_back();
                    cx.notify();
                })),
        );
    }

    let next_label = tr(lang, "下一步", "Next");
    let finish_label = tr(lang, "进入启动器", "Open launcher");
    let blocked = step == crate::state::OOBE_ACCOUNT_STEP && state.accounts.accounts.is_empty();

    bar.child(
        primary_button(if last { finish_label } else { next_label }, t)
            .id("oobe-next")
            .role(Role::Button)
            .aria_label(if last { finish_label } else { next_label })
            .when(blocked, |this| this.opacity(0.35))
            .on_click(cx.listener(|shell, _, _, cx| {
                shell.state.oobe_next();
                cx.notify();
            })),
    )
    .into_any_element()
}
