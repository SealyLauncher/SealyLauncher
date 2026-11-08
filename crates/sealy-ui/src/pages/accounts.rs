//! Accounts: three sign-in methods plus the current account.

use crate::app::SealyShell;
use crate::components::{action_bar, avatar, head_band_back, mono, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::ShellState;
use crate::theme::{self, ThemeTokens};

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band_back("04", tr(lang, "账号", "Accounts"), &t, cx))
        .child(
            div()
                .flex_1()
                .p(px(theme::PAGE_PADDING))
                .flex()
                .flex_col()
                .gap(px(22.0))
                .child(
                    div()
                        .flex()
                        .gap(px(12.0))
                        .child(method(
                            IconName::Globe,
                            "Microsoft",
                            tr(
                                lang,
                                "正版账户，可进正版服务器",
                                "Official account for online servers",
                            ),
                            &t,
                        ))
                        .child(method(
                            IconName::Server,
                            "authlib-injector",
                            tr(
                                lang,
                                "第三方验证服务器登录",
                                "Sign in through a custom auth server",
                            ),
                            &t,
                        ))
                        .child(method(
                            IconName::UserPlus,
                            tr(lang, "离线", "Offline"),
                            tr(lang, "无需联网，仅本地使用", "No network, local use only"),
                            &t,
                        )),
                )
                .child(div().w_full().h(px(1.0)).bg(rgb(t.line)))
                .child(
                    div()
                        .h(px(72.0))
                        .flex()
                        .items_center()
                        .gap(px(14.0))
                        .child(avatar("H", &t))
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
                                        .child("Haikuorenyuyue"),
                                )
                                .child(
                                    div().text_size(px(11.0)).text_color(rgb(t.muted)).child(tr(
                                        lang,
                                        "离线账号 · 8F3A",
                                        "Offline · 8F3A",
                                    )),
                                ),
                        )
                        .child(mono(tr(lang, "当前", "CURRENT"), &t)),
                ),
        )
        .child(
            action_bar(&t)
                .child(mono(tr(lang, "1 个账号", "1 account"), &t))
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "切换", "Switch"), &t)
                        .id("account-switch")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.notify(tr(
                                shell.state.language(),
                                "账号切换将在 Phase 3 接入",
                                "Switching arrives in Phase 3",
                            ));
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

fn method(icon: IconName, name: &'static str, hint: &'static str, t: &ThemeTokens) -> AnyElement {
    div()
        .flex_1()
        .h(px(128.0))
        .p(px(16.0))
        .flex()
        .flex_col()
        .justify_between()
        .rounded(px(theme::RADIUS_TILE))
        .bg(rgb(t.surface))
        .border_1()
        .border_color(rgb(t.line))
        .cursor_pointer()
        .hover(move |s| s.border_color(rgb(t.text)))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    Icon::new(icon)
                        .w(px(20.0))
                        .h(px(20.0))
                        .text_color(rgb(t.text)),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(name),
                ),
        )
        .child(
            div()
                .text_size(px(11.0))
                .line_height(relative(1.5))
                .text_color(rgb(t.muted))
                .child(hint),
        )
        .into_any_element()
}
