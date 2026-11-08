//! Home: tile dashboard with a persistent launch action.

use sealy_core::config::Language;

use crate::app::SealyShell;
use crate::components::{
    action_bar, avatar, block_tile, head_band, icon_button, list_tile, medium_tile, mono,
    primary_button, small_tile, wide_tile,
};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::{Route, ShellState};
use crate::theme::{self, ThemeTokens};

/// Instance shown until Phase 2 supplies real instance data.
const PLACEHOLDER_INSTANCE: &str = "Fabric 1.21.11";

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();

    let t_home = nav_tile(Route::Home, state.route, &t, lang, cx);
    let t_downloads = nav_tile(Route::Downloads, state.route, &t, lang, cx);
    let t_accounts = nav_tile(Route::Accounts, state.route, &t, lang, cx);
    let t_settings = nav_tile(Route::Settings, state.route, &t, lang, cx);

    let t_instances = medium_tile(IconName::Layers, tr(lang, "实例", "Instances"), &t)
        .id("tile-instances")
        .on_click(cx.listener(|shell, _, _, cx| {
            shell.state.route = Route::Instances;
            cx.notify();
        }));

    let t_install = wide_tile(
        IconName::Download,
        tr(lang, "安装新版本", "Install a version"),
        &t,
    )
    .id("tile-install")
    .on_click(cx.listener(|shell, _, _, cx| {
        shell.state.route = Route::Downloads;
        cx.notify();
    }));

    let small_grid = div()
        .flex()
        .flex_col()
        .gap(px(theme::TILE_GAP))
        .child(
            div()
                .flex()
                .gap(px(theme::TILE_GAP))
                .child(t_home)
                .child(t_downloads),
        )
        .child(
            div()
                .flex()
                .gap(px(theme::TILE_GAP))
                .child(t_accounts)
                .child(t_settings),
        );

    let left = div()
        .flex()
        .flex_col()
        .gap(px(theme::TILE_GAP))
        .child(
            div()
                .flex()
                .gap(px(theme::TILE_GAP))
                .child(small_grid)
                .child(t_instances),
        )
        .child(t_install);

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band("01", PLACEHOLDER_INSTANCE, &t))
        .child(
            div()
                .flex_1()
                .p(px(theme::PAGE_PADDING))
                .flex()
                .gap(px(theme::TILE_GAP))
                .child(left)
                .child(notice_tile(&t, lang))
                .child(worlds_tile(&t, lang)),
        )
        .child(action_bar(&t).child(action_bar_content(&t, lang, cx)))
        .into_any_element()
}

fn nav_tile(
    route: Route,
    active: Route,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let label = route.label(lang);
    small_tile(route.icon(), route == active, t)
        .id(("tile", route.index()))
        .role(Role::Button)
        .aria_label(label)
        .on_click(cx.listener(move |shell, _, _, cx| {
            shell.state.route = route;
            cx.notify();
        }))
        .into_any_element()
}

fn notice_tile(t: &ThemeTokens, lang: Language) -> impl IntoElement {
    let muted = rgba(theme::alpha(t.block_text, 0x99));
    let faint = rgba(theme::alpha(t.block_text, 0x66));

    block_tile(t)
        .id("tile-notices")
        .gap(px(14.0))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.block_text))
                .child(tr(lang, "公告", "News")),
        )
        .child(
            div()
                .text_size(px(19.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.block_text))
                .child(tr(
                    lang,
                    "1.21.11 已可安装，Fabric 0.16.5 同步更新",
                    "1.21.11 is available, Fabric 0.16.5 updated",
                )),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(notice_line(
                    "11-08",
                    tr(lang, "BMCLAPI 线路维护", "BMCLAPI maintenance"),
                    muted,
                    faint,
                ))
                .child(notice_line(
                    "11-06",
                    tr(lang, "aurora 整合包自动更新", "aurora modpack auto-update"),
                    muted,
                    faint,
                )),
        )
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(div().text_size(px(11.0)).text_color(muted).child(tr(
                    lang,
                    "全部公告",
                    "All news",
                )))
                .child(
                    Icon::new(IconName::ArrowRight)
                        .w(px(11.0))
                        .h(px(11.0))
                        .text_color(muted),
                ),
        )
}

fn notice_line(date: &str, text: &str, body: Rgba, faint: Rgba) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .pt(px(10.0))
        .border_t_1()
        .border_color(rgba(0xFFFFFF22))
        .child(
            div()
                .text_size(px(10.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(faint)
                .child(date.to_owned()),
        )
        .child(
            div()
                .text_size(px(12.0))
                .text_color(body)
                .child(text.to_owned()),
        )
}

fn worlds_tile(t: &ThemeTokens, lang: Language) -> impl IntoElement {
    list_tile(t)
        .id("tile-worlds")
        .gap(px(2.0))
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(tr(lang, "最近世界", "Recent worlds")),
                )
                .child(mono("03", t)),
        )
        .child(world_row("Aurora", "2H", t))
        .child(world_row("Skyblock", "3D", t))
        .child(world_row("Test World", "1W", t))
        .child(div().flex_1())
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .pt(px(10.0))
                .border_t_1()
                .border_color(rgb(t.line))
                .child(div().text_size(px(11.0)).text_color(rgb(t.text)).child(tr(
                    lang,
                    "查看全部",
                    "View all",
                )))
                .child(
                    Icon::new(IconName::ArrowRight)
                        .w(px(11.0))
                        .h(px(11.0))
                        .text_color(rgb(t.text)),
                ),
        )
}

fn world_row(name: &str, when: &str, t: &ThemeTokens) -> Div {
    div()
        .h(px(46.0))
        .flex()
        .items_center()
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .text_size(px(13.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(t.text))
                .child(name.to_owned()),
        )
        .child(div().w(px(8.0)))
        .child(div().flex_shrink_0().child(mono(when, t)))
}

fn action_bar_content(t: &ThemeTokens, lang: Language, cx: &mut Context<SealyShell>) -> AnyElement {
    let greet = tr(lang, "您好！", "Hi!");
    let launch = tr(lang, "启动游戏", "Launch");

    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(16.0))
        .child(avatar("H", t))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(24.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(greet),
                )
                .child(
                    div()
                        .text_size(px(24.0))
                        .text_color(rgb(t.muted))
                        .child("Haikuorenyuyue"),
                ),
        )
        .child(div().flex_1())
        .child(
            icon_button(IconName::FileText, t)
                .id("open-logs")
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell
                        .state
                        .notify(tr(lang, "日志将在 Phase 2 接入", "Logs arrive in Phase 2"));
                    cx.notify();
                })),
        )
        .child(
            primary_button(launch, t)
                .id("play")
                .role(Role::Button)
                .aria_label(launch)
                .on_click(cx.listener(move |shell, _, _, cx| {
                    shell.state.notify(tr(
                        lang,
                        "启动流程将在 Phase 2 接入",
                        "Launching arrives in Phase 2",
                    ));
                    cx.notify();
                })),
        )
        .into_any_element()
}
