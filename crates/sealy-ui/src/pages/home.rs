//! Home: tile dashboard with a persistent launch action.

use sealy_core::config::{Language, MotionSetting};
use std::time::Duration;

use crate::app::SealyShell;
use crate::components::{
    action_bar, block_tile, icon_button, index_badge, list_tile, medium_tile, mono, primary_button,
    small_tile, wide_tile,
};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::{Route, ShellState};
use crate::theme::{self, ThemeTokens};

/// Instance shown until Phase 2 supplies real instance data.
const PLACEHOLDER_INSTANCE: &str = "Fabric 1.21.11";
/// Instance metadata line under the name.
const PLACEHOLDER_META: &str = "1.21.11 · Fabric 0.16.5";
/// Account name placeholder until Phase 3.
const PLACEHOLDER_USER: &str = "Haikuorenyuyue";

pub(crate) fn render(state: &ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    let t = state.theme();
    let lang = state.language();
    let (instance_name, instance_meta) = match state.instances.current() {
        Some(instance) => (
            instance.name.clone(),
            if instance.name == instance.version_id {
                tr(lang, "独立存档", "Separate saves").to_owned()
            } else {
                instance.version_id.clone()
            },
        ),
        None => (PLACEHOLDER_INSTANCE.to_owned(), PLACEHOLDER_META.to_owned()),
    };

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
        .child(head_band_carousel(&t, lang, state.settings.motion))
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
        .child(action_bar(&t).child(action_bar_content(
            &t,
            lang,
            &instance_name,
            &instance_meta,
            cx,
        )))
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

fn action_bar_content(
    t: &ThemeTokens,
    lang: Language,
    instance: &str,
    meta: &str,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let launch = tr(lang, "启动游戏", "Launch");

    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(16.0))
        .child(instance_block(t, instance, meta))
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
                    crate::launch_flow::start_selected(&mut shell.state, cx);
                })),
        )
        .into_any_element()
}

/// Head band whose title crossfades between the launcher name and the signed-in
/// greeting on an eight-second loop. When motion is disabled the launcher name
/// stays put.
fn head_band_carousel(t: &ThemeTokens, lang: Language, motion: MotionSetting) -> Div {
    let _ = lang;

    let mut band = div()
        .h(px(theme::HEAD_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(14.0))
        .px(px(theme::PAGE_PADDING))
        .bg(rgb(t.surface))
        .border_b_1()
        .border_color(rgb(t.line))
        .child(index_badge("01", t));

    if motion == MotionSetting::Never {
        return band.child(
            div()
                .text_size(px(42.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child("SealyLauncher"),
        );
    }

    let clock = Animation::new(Duration::from_secs(8)).repeat_synced();
    let greeting = format!("Hi，{PLACEHOLDER_USER}");

    band = band.child(
        div()
            .relative()
            .flex_1()
            .h(px(56.0))
            .child(title_slide(
                "SealyLauncher".to_owned(),
                true,
                clock.clone(),
                t,
            ))
            .child(title_slide(greeting, false, clock, t)),
    );
    band
}

fn title_slide(text: String, first: bool, clock: Animation, t: &ThemeTokens) -> impl IntoElement {
    let id = if first { "head-brand" } else { "head-greeting" };
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .child(
            div()
                .text_size(px(42.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.text))
                .child(text),
        )
        .with_animation(id, clock, move |el, delta| {
            let seen = slide_opacity(delta, first);
            el.opacity(seen).mt(px((1.0 - seen) * 6.0))
        })
}

/// The instance the play button will launch.
fn instance_block(t: &ThemeTokens, name: &str, meta: &str) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(12.0))
        .child(
            div()
                .w(px(42.0))
                .h(px(42.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(theme::RADIUS_TILE))
                .border_1()
                .border_color(rgb(t.line))
                .child(
                    Icon::new(IconName::Layers)
                        .w(px(20.0))
                        .h(px(20.0))
                        .text_color(rgb(t.text)),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(20.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(t.text))
                        .child(name.to_owned()),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(rgb(t.muted))
                        .child(meta.to_owned()),
                ),
        )
}

/// Opacity of one slide during the shared eight-second loop.
///
/// The eight-second cycle splits into two four-second halves; each half fades
/// in, holds, then fades out, so two titles are never visible at once.
fn slide_opacity(delta: f32, first: bool) -> f32 {
    const FADE: f32 = 0.06;
    let (start, end) = if first { (0.0, 0.5) } else { (0.5, 1.0) };
    if !(start..end).contains(&delta) {
        return 0.0;
    }
    let rise = ((delta - start) / FADE).clamp(0.0, 1.0);
    let fall = ((end - delta) / FADE).clamp(0.0, 1.0);
    rise.min(fall)
}
