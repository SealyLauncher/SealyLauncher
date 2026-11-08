//! Downloads: live version list from the manifest, grouped by channel.

use sealy_core::config::Language;
use sealy_core::manifest::{VersionKind, VersionManifest, VersionSummary};

use crate::app::SealyShell;
use crate::components::{action_bar, head_band_back, mono, quiet_button};
use crate::i18n::tr;
use crate::kit::*;
use crate::state::{InstallTask, MessageTone, ShellState, VersionFilter};
use crate::theme::{self, ThemeTokens};

/// Rows rendered per screen batch; the manifest holds hundreds of versions.
const MAX_ROWS: usize = 60;

pub(crate) fn render(state: &mut ShellState, cx: &mut Context<SealyShell>) -> AnyElement {
    ensure_loaded(state, cx);

    let t = state.theme();
    let lang = state.language();
    let filter = state.downloads.filter;

    let mut list = div()
        .id("version-list")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scroll()
        .flex()
        .flex_col();
    if let Some(manifest) = state.downloads.manifest.clone() {
        let rows = manifest
            .versions
            .iter()
            .filter(|version| matches(filter, version.kind))
            .take(MAX_ROWS)
            .enumerate()
            .map(|(index, version)| row(index, version, lang, &t, cx))
            .collect::<Vec<_>>();
        list = list.children(rows);
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head_band_back("03", tr(lang, "下载", "Downloads"), &t, cx))
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .p(px(theme::PAGE_PADDING))
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(44.0))
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .h(px(36.0))
                                .w(px(260.0))
                                .px(px(12.0))
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .rounded(px(theme::RADIUS_TILE))
                                .border_1()
                                .border_color(rgb(t.line))
                                .child(
                                    Icon::new(IconName::Search)
                                        .w(px(14.0))
                                        .h(px(14.0))
                                        .text_color(rgb(t.muted)),
                                )
                                .child(
                                    div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
                                        lang,
                                        "搜索版本",
                                        "Search versions",
                                    )),
                                ),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("filter")
                                .role(Role::Button)
                                .aria_label("filter")
                                .cursor_pointer()
                                .on_click(cx.listener(|shell, _, _, cx| {
                                    shell.state.downloads.filter =
                                        shell.state.downloads.filter.next();
                                    cx.notify();
                                }))
                                .child(mono(
                                    match filter {
                                        VersionFilter::All => tr(lang, "全部", "All"),
                                        VersionFilter::Release => tr(lang, "正式版", "Releases"),
                                        VersionFilter::Snapshot => tr(lang, "快照", "Snapshots"),
                                    },
                                    &t,
                                )),
                        ),
                )
                .child(div().w_full().h(px(1.0)).bg(rgb(t.line)))
                .child(status(state, &t, lang, cx))
                .child(list),
        )
        .child(
            action_bar(&t)
                .child(mono(count_label(state, lang), &t))
                .child(div().flex_1())
                .child(
                    quiet_button(tr(lang, "刷新", "Refresh"), &t)
                        .id("manifest-refresh")
                        .on_click(cx.listener(|shell, _, _, cx| {
                            shell.state.downloads.loading = false;
                            shell.state.downloads.manifest = None;
                            shell.state.downloads.error = None;
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

fn count_label(state: &ShellState, lang: Language) -> String {
    match &state.downloads.manifest {
        Some(manifest) => format!(
            "{} {}",
            manifest.versions.len(),
            tr(lang, "个版本", "versions")
        ),
        None => tr(lang, "尚无数据", "no data").to_owned(),
    }
}

fn status(
    state: &ShellState,
    t: &ThemeTokens,
    lang: Language,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    if state.downloads.loading {
        return div()
            .h(px(72.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .child(div().w(px(12.0)).h(px(12.0)).rounded_full().bg(rgb(t.text)))
            .child(div().text_size(px(12.0)).text_color(rgb(t.muted)).child(tr(
                lang,
                "正在获取版本清单…",
                "Fetching the version list…",
            )))
            .into_any_element();
    }

    if let Some(error) = &state.downloads.error {
        return div()
            .h(px(96.0))
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(rgb(t.line_soft))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(t.text))
                            .child(tr(
                                lang,
                                "无法获取版本清单",
                                "Could not fetch the version list",
                            )),
                    )
                    .child(
                        div()
                            .max_w(px(640.0))
                            .text_size(px(11.0))
                            .text_color(rgb(t.muted))
                            .child(error.clone()),
                    ),
            )
            .child(
                quiet_button(tr(lang, "重试", "Retry"), t)
                    .id("manifest-retry")
                    .on_click(cx.listener(|shell, _, _, cx| {
                        shell.state.downloads.loading = false;
                        shell.state.downloads.error = None;
                        cx.notify();
                    })),
            )
            .into_any_element();
    }

    if state.downloads.manifest.is_some() {
        return div().h(px(8.0)).into_any_element();
    }

    div().h(px(8.0)).into_any_element()
}

fn ensure_loaded(state: &mut ShellState, cx: &mut Context<SealyShell>) {
    if state.downloads.loading
        || state.downloads.manifest.is_some()
        || state.downloads.error.is_some()
    {
        return;
    }

    state.downloads.loading = true;
    let policy = state.settings.download.metadata_source;

    cx.spawn(async move |this, cx| {
        let result = cx
            .background_executor()
            .spawn(async move { sealy_core::manifest::fetch_manifest(policy) })
            .await;

        this.update(cx, |shell, cx| {
            shell.state.downloads.loading = false;
            match result {
                Ok(manifest) => shell.state.downloads.manifest = Some(manifest),
                Err(err) => shell.state.downloads.error = Some(err.to_string()),
            }
            cx.notify();
        })
        .ok();
    })
    .detach();
}

fn matches(filter: VersionFilter, kind: VersionKind) -> bool {
    match filter {
        VersionFilter::All => true,
        VersionFilter::Release => kind == VersionKind::Release,
        VersionFilter::Snapshot => kind.is_snapshot(),
    }
}

fn row(
    index: usize,
    version: &VersionSummary,
    lang: Language,
    t: &ThemeTokens,
    cx: &mut Context<SealyShell>,
) -> AnyElement {
    div()
        .h(px(60.0))
        .flex()
        .items_center()
        .gap(px(16.0))
        .border_b_1()
        .border_color(rgb(t.line_soft))
        .child(
            div()
                .w(px(150.0))
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(t.text))
                .child(version.id.clone()),
        )
        .child(
            div()
                .w(px(72.0))
                .text_size(px(11.0))
                .text_color(rgb(t.muted))
                .child(match version.kind {
                    VersionKind::Release => tr(lang, "正式版", "Release"),
                    VersionKind::Snapshot => tr(lang, "快照", "Snapshot"),
                    VersionKind::OldAlpha => tr(lang, "远古 Alpha", "Old alpha"),
                    VersionKind::OldBeta => tr(lang, "远古 Beta", "Old beta"),
                }),
        )
        .child(div().flex_1())
        .child(mono(VersionManifest::date_of(version), t))
        .child(mono(format!("#{:03}", index + 1), t))
        .child(
            quiet_button(tr(lang, "安装", "Install"), t)
                .id(("install", index))
                .role(Role::Button)
                .aria_label(format!("install {}", version.id))
                .on_click(cx.listener({
                    let id = version.id.clone();
                    move |shell, _, _, cx| {
                        start_install(&mut shell.state, id.clone(), cx);
                    }
                })),
        )
        .into_any_element()
}

/// Kicks off a background install and reports into the message center.
fn start_install(state: &mut ShellState, id: String, cx: &mut Context<SealyShell>) {
    if state.install.is_some() {
        return;
    }

    let root = state.paths.root.clone();
    let policy = state.settings.download.metadata_source;
    let concurrency = usize::from(state.settings.download.concurrency).max(1);
    let lang = state.language();

    let message_id = state.push_message(
        format!("安装 {id}"),
        tr(lang, "正在获取版本文件…", "Fetching version files…").to_owned(),
        MessageTone::Running,
    );
    state.message_open = true;

    let handle = std::sync::Arc::new(std::sync::Mutex::new(
        sealy_core::install::Progress::default(),
    ));
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    state.install = Some(InstallTask {
        message_id,
        progress: handle.clone(),
        cancel: cancel.clone(),
    });

    let worker = handle.clone();
    let worker_cancel = cancel.clone();
    let task_id = id.clone();

    cx.spawn(async move |this, cx| {
        let task = cx.background_executor().spawn(async move {
            sealy_core::install::install(
                &root,
                &task_id,
                policy,
                concurrency,
                worker,
                worker_cancel,
            )
        });

        loop {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(200))
                .await;

            let snapshot = handle
                .lock()
                .map(|progress| progress.clone())
                .unwrap_or_default();
            let finished = snapshot.finished;

            let _ = this.update(cx, |shell, cx| {
                let filler = if snapshot.total == 0 {
                    tr(
                        shell.state.language(),
                        "正在获取版本文件…",
                        "Fetching version files…",
                    )
                    .to_owned()
                } else {
                    format!(
                        "{} / {} · {}",
                        snapshot.done,
                        snapshot.total,
                        snapshot
                            .current
                            .rsplit('/')
                            .next()
                            .unwrap_or(&snapshot.current)
                    )
                };
                let ratio = if snapshot.total == 0 {
                    0.0
                } else {
                    (snapshot.done as f32 / snapshot.total as f32).clamp(0.0, 1.0)
                };

                shell.state.update_message(message_id, |row| {
                    row.progress = Some(ratio);
                    row.detail = filler;
                });
                cx.notify();
            });

            if finished {
                break;
            }
        }

        match task.await {
            Ok(installed) => {
                let _ = this.update(cx, |shell, cx| {
                    shell.state.install = None;
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Done;
                        row.progress = Some(1.0);
                        row.detail = format!(
                            "{} 个文件 · {} MB",
                            installed.files,
                            installed.bytes / 1_048_576
                        );
                    });
                    cx.notify();
                });
            }
            Err(err) => {
                let _ = this.update(cx, |shell, cx| {
                    shell.state.install = None;
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Failed;
                        row.progress = None;
                        row.detail = err.to_string();
                    });
                    cx.notify();
                });
            }
        }
    })
    .detach();
}
