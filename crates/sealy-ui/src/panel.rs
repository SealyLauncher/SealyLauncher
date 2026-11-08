//! Global message center: background tasks and their outcome.

use crate::app::SealyShell;
use crate::components::mono;
use crate::i18n::tr;
use crate::kit::*;
use crate::state::{MessageTone, ShellState};
use crate::theme::{self, ThemeTokens};

/// Slide-over panel anchored to the right edge of the content area.
pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let rows = state
        .messages
        .iter()
        .map(|message| row(message, &t, cx))
        .collect::<Vec<_>>();

    let body = if rows.is_empty() {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(8.0))
            .child(
                Icon::new(IconName::Inbox)
                    .w(px(26.0))
                    .h(px(26.0))
                    .text_color(rgb(t.faint)),
            )
            .child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
                lang,
                "暂无消息",
                "No messages yet",
            )))
            .into_any_element()
    } else {
        div()
            .id("message-list")
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .children(rows)
            .into_any_element()
    };

    div()
        .absolute()
        .top(px(0.0))
        .right(px(0.0))
        .bottom(px(0.0))
        .w(px(380.0))
        .flex()
        .flex_col()
        .bg(rgb(t.surface))
        .border_l_1()
        .border_color(rgb(t.line))
        .child(
            div()
                .h(px(56.0))
                .flex_shrink_0()
                .px(px(20.0))
                .flex()
                .items_center()
                .gap(px(10.0))
                .border_b_1()
                .border_color(rgb(t.line))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(tr(lang, "消息中心", "Message center")),
                )
                .child(mono(format!("{:02}", state.messages.len()), &t))
                .child(
                    div()
                        .id("messages-close")
                        .role(Role::Button)
                        .aria_label("close messages")
                        .w(px(28.0))
                        .h(px(28.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(theme::RADIUS_TILE))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgb(t.line_soft)))
                        .child(
                            Icon::new(IconName::X)
                                .w(px(15.0))
                                .h(px(15.0))
                                .text_color(rgb(t.muted)),
                        )
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.message_open = false;
                            cx.notify();
                        })),
                ),
        )
        .child(body)
        .into_any_element()
}

fn row(
    message: &crate::state::Message,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let (icon, tone_color) = match message.tone {
        MessageTone::Running => (IconName::Loader, t.text),
        MessageTone::Done => (IconName::Check, t.text),
        MessageTone::Failed => (IconName::CircleAlert, t.danger),
    };
    let message_id = message.id;

    let mut card = div()
        .p(px(16.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    Icon::new(icon)
                        .w(px(14.0))
                        .h(px(14.0))
                        .text_color(rgb(tone_color)),
                )
                .child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(t.text))
                        .child(message.title.clone()),
                )
                .when(message.tone == MessageTone::Running, |this| {
                    this.child(
                        div()
                            .id(("cancel", message_id as usize))
                            .role(Role::Button)
                            .aria_label(format!("cancel {message_id}"))
                            .px(px(8.0))
                            .h(px(22.0))
                            .flex()
                            .items_center()
                            .rounded(px(theme::RADIUS_TILE))
                            .border_1()
                            .border_color(rgb(t.line))
                            .cursor_pointer()
                            .text_size(px(10.0))
                            .text_color(rgb(t.muted))
                            .child("取消")
                            .on_click(cx.listener(move |shell, _, _, cx| {
                                if let Some(task) = &shell.state.install {
                                    task.cancel
                                        .store(true, std::sync::atomic::Ordering::Relaxed);
                                }
                                shell.state.update_message(message_id, |row| {
                                    row.detail = "正在取消…".to_owned();
                                });
                                cx.notify();
                            })),
                    )
                }),
        )
        .child(
            div()
                .text_size(px(11.0))
                .line_height(relative(1.5))
                .text_color(rgb(t.muted))
                .child(message.detail.clone()),
        );

    if let Some(progress) = message.progress {
        card = card.child(
            div()
                .mt(px(4.0))
                .w_full()
                .h(px(3.0))
                .rounded(px(2.0))
                .bg(rgb(t.line_soft))
                .child(
                    div()
                        .w(relative(progress))
                        .h_full()
                        .rounded(px(2.0))
                        .bg(rgb(t.text)),
                ),
        );
    }

    card.into_any_element()
}
