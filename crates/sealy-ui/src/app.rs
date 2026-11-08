//! Phase 0 application shell: a GPUI window with a custom title bar,
//! layered background, and an animated page switch.

use gpui_kit::component::TitleBar;
use gpui_kit::prelude::*;
use gpui_kit::{
    Animation, AnimationExt, Bounds, Context, ElementId, FontWeight, IntoElement, Render, Window,
    WindowBounds, WindowOptions, div, px, rgb, size,
};

use crate::motion;
use crate::theme;

/// Starts the GPUI application and opens the Phase 0 shell window.
pub fn run() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);

            let bounds = Bounds::centered(None, size(px(1180.0), px(760.0)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(960.0), px(640.0))),
                ..TitleBar::window_options()
            };

            gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| SealyShell::new()))
                .expect("failed to open the SealyLauncher window");
        });
}

struct SealyShell {
    page: usize,
}

impl SealyShell {
    fn new() -> Self {
        Self { page: 0 }
    }

    fn page_copy(&self) -> (&'static str, &'static str, &'static str) {
        if self.page == 0 {
            (
                "SealyLauncher",
                "Rust + GPUI · portable Minecraft launcher",
                "切换页面",
            )
        } else {
            (
                "Phase 0 · Motion",
                "240ms ease-out · opacity + 14px lift",
                "返回首页",
            )
        }
    }
}

impl Render for SealyShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.page;
        let (title, caption, action) = self.page_copy();
        let page_id = ElementId::named_usize("page-card", page);

        let content = div()
            .id("page-card")
            .flex()
            .flex_col()
            .items_center()
            .gap_5()
            .child(
                div()
                    .text_size(px(48.0))
                    .font_weight(FontWeight::BOLD)
                    .child(title),
            )
            .child(div().text_sm().text_color(rgb(theme::MUTED)).child(caption))
            .child(
                div()
                    .id("toggle-page")
                    .px_5()
                    .py_3()
                    .rounded_xl()
                    .bg(rgb(theme::SURFACE))
                    .border_1()
                    .border_color(rgb(theme::BORDER))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child(action),
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.page = 1 - this.page;
                        cx.notify();
                    })),
            );

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(theme::CANVAS))
            .text_color(rgb(theme::TEXT))
            .child(
                TitleBar::new().child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .child(
                            div()
                                .w(px(16.0))
                                .h(px(16.0))
                                .rounded_full()
                                .bg(rgb(theme::ACCENT)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!("SealyLauncher v{}", sealy_core::version())),
                        ),
                ),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .bg(rgb(theme::CANVAS))
                    .child(
                        div()
                            .absolute()
                            .top(px(-140.0))
                            .left(px(-100.0))
                            .size(px(560.0))
                            .rounded_full()
                            .bg(rgb(theme::ACCENT_DEEP))
                            .opacity(0.42),
                    )
                    .child(
                        div()
                            .absolute()
                            .bottom(px(-200.0))
                            .right(px(-120.0))
                            .size(px(680.0))
                            .rounded_full()
                            .bg(rgb(theme::TEAL_DEEP))
                            .opacity(0.35),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(90.0))
                            .right(px(140.0))
                            .size(px(300.0))
                            .rounded_full()
                            .bg(rgb(theme::ACCENT))
                            .opacity(0.07),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(0.0))
                            .left(px(0.0))
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(content.with_animation(
                                page_id,
                                Animation::new(motion::PAGE).with_easing(motion::ease_out_cubic),
                                |el, delta| el.opacity(delta).mt(px((1.0 - delta) * 14.0)),
                            )),
                    ),
            )
    }
}
