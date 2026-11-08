//! Phase 1 application shell: title bar, navigation rail, route transitions.

use gpui_kit::component::TitleBar;

use crate::components;
use crate::kit::*;
use crate::motion;
use crate::pages;
use crate::state::{Route, ShellState};
use crate::theme;

/// Starts the GPUI application and opens the shell window.
pub fn run(bootstrap: sealy_core::Bootstrap) {
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

            gpui_kit::open_window(options, cx, move |_, cx| {
                cx.new(|_| SealyShell::new(bootstrap))
            })
            .expect("failed to open the SealyLauncher window");
        });
}

pub(crate) struct SealyShell {
    pub(crate) state: ShellState,
}

impl SealyShell {
    fn new(bootstrap: sealy_core::Bootstrap) -> Self {
        Self {
            state: ShellState::from_bootstrap(bootstrap),
        }
    }
}

impl Render for SealyShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.state.language();
        let route = self.state.route;
        let page = pages::render(&self.state, cx);
        let page_id = ElementId::named_usize("route-page", route.index());

        let nav_items = Route::ALL
            .iter()
            .map(|route| nav_item(*route, self.state.route, lang, cx))
            .collect::<Vec<_>>();

        let notice = self
            .state
            .notice
            .as_ref()
            .map(|message| toast(message.clone(), cx));

        div()
            .id("shell")
            .role(Role::Application)
            .aria_label("SealyLauncher")
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
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .child(nav_rail(nav_items))
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(components::background_stage())
                            .child(
                                div()
                                    .absolute()
                                    .top(px(0.0))
                                    .left(px(0.0))
                                    .size_full()
                                    .child(
                                        div().child(page).with_animation(
                                            page_id,
                                            Animation::new(motion::PAGE)
                                                .with_easing(motion::ease_out_cubic),
                                            |el, delta| {
                                                el.opacity(delta).mt(px((1.0 - delta) * 10.0))
                                            },
                                        ),
                                    ),
                            )
                            .children(notice),
                    ),
            )
    }
}

fn nav_rail(items: Vec<AnyElement>) -> Div {
    div()
        .w(px(theme::NAV_WIDTH))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .py_3()
        .bg(rgba(0x0B0D12D9))
        .border_1()
        .border_color(rgba(0xFFFFFF0A))
        .child(
            div()
                .w(px(36.0))
                .h(px(36.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(theme::RADIUS_MD))
                .bg(rgb(theme::ACCENT))
                .child(
                    div()
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x06121A))
                        .child("S"),
                ),
        )
        .child(div().h(px(18.0)))
        .children(items)
        .child(div().flex_1())
        .child(
            div()
                .w(px(36.0))
                .h(px(36.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(rgb(theme::SURFACE_RAISED))
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::ACCENT))
                        .child("S"),
                ),
        )
}

fn nav_item(
    route: Route,
    active_route: Route,
    lang: sealy_core::config::Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    let active = route == active_route;
    let label = route.label(lang);

    div()
        .id(("nav", route.index()))
        .role(Role::Button)
        .aria_label(label)
        .w(px(44.0))
        .h(px(44.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_MD))
        .cursor_pointer()
        .when(active, |this| {
            this.bg(rgba(0x4CC2FF1F))
                .border_1()
                .border_color(rgba(0x4CC2FF3D))
        })
        .child(
            Icon::new(route.icon())
                .w(px(19.0))
                .h(px(19.0))
                .text_color(if active {
                    rgb(theme::ACCENT)
                } else {
                    rgb(theme::MUTED)
                }),
        )
        .on_click(cx.listener(move |shell, _, _, cx| {
            shell.state.route = route;
            cx.notify();
        }))
        .into_any_element()
}

fn toast(message: String, cx: &mut Context<SealyShell>) -> AnyElement {
    components::glass_panel()
        .id("notice")
        .role(Role::Button)
        .aria_label(message.clone())
        .absolute()
        .bottom(px(24.0))
        .right(px(24.0))
        .px_4()
        .py_3()
        .cursor_pointer()
        .on_click(cx.listener(|shell, _, _, cx| {
            shell.state.notice = None;
            cx.notify();
        }))
        .child(
            div()
                .text_size(px(12.0))
                .text_color(rgb(theme::TEXT))
                .child(message),
        )
        .into_any_element()
}
