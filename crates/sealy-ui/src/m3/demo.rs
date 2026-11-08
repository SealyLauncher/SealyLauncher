//! Material 3 prototype window.
//!
//! Purpose: validate palette derivation, M3 widgets, ripple and state layers
//! inside GPUI before rewriting `sealy-ui`.

use crate::kit::*;
use crate::m3::palette::{M3Scheme, scheme};
use crate::m3::widgets;

const SEEDS: [(u32, &str); 4] = [
    (0x6750A4, "紫"),
    (0x006A6A, "青"),
    (0x8B5000, "橙"),
    (0x005AC1, "蓝"),
];

/// Opens the prototype window.
pub fn run() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);

            let bounds = Bounds::centered(None, size(px(920.0), px(620.0)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(860.0), px(580.0))),
                ..Default::default()
            };

            gpui_kit::open_window(options, cx, |window, cx| {
                window.set_window_title("SealyLauncher · Material 3");
                cx.new(|_| M3Demo::new())
            })
            .expect("failed to open M3 prototype window");
        });
}

struct M3Demo {
    seed: u32,
    dark: bool,
    nav: usize,
    ripples: usize,
}

impl M3Demo {
    fn new() -> Self {
        Self {
            seed: SEEDS[0].0,
            dark: true,
            nav: 0,
            ripples: 0,
        }
    }
}

impl Render for M3Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s: M3Scheme = scheme(self.seed, self.dark);

        div()
            .size_full()
            .flex()
            .bg(rgb(s.surface))
            .text_color(rgb(s.on_surface))
            .font_weight(FontWeight::NORMAL)
            .child(self.rail(&s, cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(self.top_bar(&s, cx))
                    .child(self.content(&s, cx))
                    .child(
                        div().absolute().right(px(24.0)).bottom(px(16.0)).child(
                            widgets::fab(IconName::Plus, &s)
                                .id("m3-fab")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.ripples += 1;
                                    cx.notify();
                                })),
                        ),
                    ),
            )
    }
}

impl M3Demo {
    fn rail(&self, s: &M3Scheme, cx: &mut Context<Self>) -> Div {
        let items = [
            (IconName::House, "主页"),
            (IconName::FolderClosed, "实例"),
            (IconName::Download, "下载"),
            (IconName::CircleUser, "账号"),
        ];

        let nav = items
            .iter()
            .enumerate()
            .map(|(index, (name, label))| {
                widgets::nav_item(*name, *label, index == self.nav, s)
                    .id(("m3-nav", index))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.nav = index;
                        cx.notify();
                    }))
            })
            .collect::<Vec<_>>();

        div()
            .w(px(88.0))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(4.0))
            .py(px(16.0))
            .bg(rgb(s.surface))
            .border_r_1()
            .border_color(rgb(s.surface_container_high))
            .child(
                div()
                    .w(px(40.0))
                    .h(px(40.0))
                    .mb(px(16.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(12.0))
                    .bg(rgb(s.primary_container))
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(s.on_primary_container))
                            .child("S"),
                    ),
            )
            .children(nav)
            .child(div().flex_1())
            .child(
                widgets::nav_item(IconName::Settings2, "设置", false, s)
                    .id("m3-nav-settings")
                    .on_click(cx.listener(|_, _, _, _| {})),
            )
            .child(
                div()
                    .mt(px(8.0))
                    .w(px(40.0))
                    .h(px(40.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(999.0))
                    .bg(rgb(s.tertiary_container))
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(s.on_tertiary_container))
                            .child("T"),
                    ),
            )
    }

    fn top_bar(&self, s: &M3Scheme, cx: &mut Context<Self>) -> Div {
        let seed_chips = SEEDS
            .iter()
            .enumerate()
            .map(|(index, (value, label))| {
                let selected = self.seed == *value;
                div()
                    .id(("m3-seed", index))
                    .w(px(30.0))
                    .h(px(30.0))
                    .rounded(px(999.0))
                    .bg(rgb(*value))
                    .border_2()
                    .border_color(if selected {
                        rgb(s.on_surface)
                    } else {
                        rgba(0x00000000)
                    })
                    .cursor_pointer()
                    .child(div())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.seed = *value;
                        this.ripples += 1;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .text_size(px(0.0))
                            .text_color(rgba(0x00000000))
                            .child(*label),
                    )
            })
            .collect::<Vec<_>>();

        div()
            .h(px(64.0))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(12.0))
            .px(px(24.0))
            .child(
                div()
                    .text_size(px(20.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(s.on_surface))
                    .child("SealyLauncher"),
            )
            .child(div().flex_1())
            .child(
                div()
                    .h(px(40.0))
                    .w(px(260.0))
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .px(px(16.0))
                    .rounded(px(999.0))
                    .bg(rgb(s.surface_container_high))
                    .child(
                        Icon::new(IconName::Search)
                            .w(px(16.0))
                            .h(px(16.0))
                            .text_color(rgb(s.on_surface_variant)),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(s.on_surface_variant))
                            .child("搜索实例、版本或服务器"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .children(seed_chips),
            )
            .child(
                widgets::icon_button(
                    if self.dark {
                        IconName::Sun
                    } else {
                        IconName::Moon
                    },
                    s,
                )
                .id("m3-theme")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.dark = !this.dark;
                    cx.notify();
                })),
            )
    }

    fn content(&self, s: &M3Scheme, cx: &mut Context<Self>) -> Div {
        let play_ripple = widgets::ripple(self.ripples, s.on_primary);

        let cards = [
            (IconName::CloudDownload, "安装新版本", "官方源 · 1.21.4"),
            (IconName::FolderClosed, "导入整合包", ".zip · .mrpack"),
            (IconName::Settings2, "管理 Java", "3 个运行时"),
        ]
        .iter()
        .map(|(name, title, caption)| {
            widgets::card(s)
                .flex_1()
                .gap(px(8.0))
                .child(widgets::tonal_bubble(*name, s))
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(s.on_surface))
                        .child(*title),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(rgb(s.on_surface_variant))
                        .child(*caption),
                )
        })
        .collect::<Vec<_>>();

        div()
            .flex_1()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(10.0))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(s.primary))
                            .child("已就绪 · 上次游玩 2 小时前"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .text_size(px(40.0))
                                    .font_weight(FontWeight::NORMAL)
                                    .text_color(rgb(s.on_surface))
                                    .child("Aurora"),
                            )
                            .child(
                                div()
                                    .text_size(px(40.0))
                                    .font_weight(FontWeight::NORMAL)
                                    .text_color(rgb(s.on_surface_variant))
                                    .child("生存服"),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(13.5))
                            .text_color(rgb(s.on_surface_variant))
                            .child("1.20.1 · Fabric 0.16.5 · 8 个模组 · Java 17 · 4096 MB"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(widgets::chip("正式版", false, s))
                    .child(widgets::chip("Fabric", true, s))
                    .child(widgets::chip("离线可用", false, s)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .relative()
                            .overflow_hidden()
                            .rounded(px(999.0))
                            .child(
                                widgets::filled_button("开始游戏", Some(IconName::Play), s)
                                    .id("m3-play")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.ripples += 1;
                                        cx.notify();
                                    })),
                            )
                            .child(play_ripple),
                    )
                    .child(
                        widgets::outlined_button("设置", s)
                            .id("m3-settings")
                            .on_click(cx.listener(|_, _, _, _| {})),
                    ),
            )
            .child(div().flex().gap(px(12.0)).children(cards))
            .child(
                widgets::card(s)
                    .py(px(13.0))
                    .gap(px(9.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(rgb(s.on_surface))
                                    .child("正在校验资源文件"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(s.primary))
                                    .child("38%"),
                            ),
                    )
                    .child(widgets::linear_progress(0.38, s)),
            )
            .child(div().flex_1())
            .child(palette_strip(s, cx))
    }
}

/// Row of role swatches, to verify the generated scheme at a glance.
fn palette_strip(s: &M3Scheme, cx: &mut Context<M3Demo>) -> Div {
    let roles = [
        ("primary", s.primary),
        ("on-primary", s.on_primary),
        ("container", s.primary_container),
        ("surface", s.surface),
        ("low", s.surface_container_low),
        ("container", s.surface_container),
        ("high", s.surface_container_high),
        ("highest", s.surface_container_highest),
        ("outline", s.outline),
        ("variant", s.outline_variant),
    ];

    let swatches = roles
        .iter()
        .map(|(label, color)| {
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .h(px(20.0))
                        .w(px(52.0))
                        .rounded(px(5.0))
                        .bg(rgb(*color))
                        .border_1()
                        .border_color(rgb(s.outline_variant)),
                )
                .child(
                    div()
                        .text_size(px(9.0))
                        .text_color(rgb(s.on_surface_variant))
                        .child(*label),
                )
        })
        .collect::<Vec<_>>();

    div()
        .flex()
        .items_end()
        .gap(px(8.0))
        .pr(px(84.0))
        .child(
            div()
                .id("m3-palette")
                .flex()
                .gap(px(8.0))
                .children(swatches)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.ripples += 1;
                    cx.notify();
                })),
        )
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(10.0))
                .text_color(rgb(s.on_surface_variant))
                .child(format!("seed #{:06X}", s.seed)),
        )
}
