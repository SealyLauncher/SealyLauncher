//! Shared install flow: starts a version install from any page.

use crate::app::SealyShell;
use crate::i18n::tr;
use crate::kit::Context;
use crate::state::{InstallTask, MessageTone, ShellState};

/// Installs `id`, reporting progress into the message center.
pub(crate) fn start(state: &mut ShellState, id: String, cx: &mut Context<SealyShell>) {
    if state.install.is_some() {
        return;
    }

    let game = state.paths.game.clone();
    let policy = state.settings.download.metadata_source;
    let concurrency = usize::from(state.settings.download.concurrency).max(1);
    let auto_java = state.settings.java.auto_download;
    let lang = state.language();

    let message_id = state.push_message(
        format!("安装 {id}"),
        tr(lang, "正在获取版本文件…", "Fetching version files…").to_owned(),
        MessageTone::Running,
    );
    state.message_open = true;
    state.update_message(message_id, |row| row.retry = Some(id.clone()));

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
    let java_dir = game.clone();
    let files_dir = game.clone();

    cx.spawn(async move |this, cx| {
        let task = cx.background_executor().spawn(async move {
            sealy_core::install::install(
                &files_dir,
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
                    let name = snapshot
                        .current
                        .rsplit('/')
                        .next()
                        .unwrap_or(&snapshot.current);
                    match (snapshot.current_bytes * 100).checked_div(snapshot.current_size) {
                        Some(percent) => {
                            format!("{name} · {percent}% · {}/{}", snapshot.done, snapshot.total)
                        }
                        None => format!("{} / {} · {name}", snapshot.done, snapshot.total),
                    }
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

        let installed = match task.await {
            Ok(installed) => installed,
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
                return;
            }
        };

        let summary = if installed.bytes == 0 {
            format!("{} 个文件 · 已是最新", installed.files)
        } else {
            format!(
                "{} 个文件 · {} MB",
                installed.files,
                installed.bytes / 1_048_576
            )
        };
        let fresh = installed.bytes > 0;
        let retry_id = id.clone();
        let _ = this.update(cx, |shell, cx| {
            shell.state.update_message(message_id, |row| {
                row.tone = MessageTone::Done;
                row.progress = Some(1.0);
                row.detail = summary.clone();
                row.retry = fresh.then(|| retry_id.clone());
            });
            cx.notify();
        });

        // Phase two: a matching Java runtime.
        let Some((component, major)) = installed.java.filter(|_| auto_java) else {
            let _ = this.update(cx, |shell, cx| {
                shell.state.install = None;
                cx.notify();
            });
            return;
        };

        let java_message = {
            let mut created = 0;
            let _ = this.update(cx, |shell, cx| {
                created = shell.state.push_message(
                    format!("准备 Java {major}"),
                    tr(
                        shell.state.language(),
                        "正在检测本机 Java…",
                        "Looking for a local Java…",
                    )
                    .to_owned(),
                    MessageTone::Running,
                );
                shell.state.message_open = true;
                cx.notify();
            });
            created
        };

        let java_progress = std::sync::Arc::new(std::sync::Mutex::new(
            sealy_core::install::Progress::default(),
        ));
        let java_cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let java_worker = java_progress.clone();
        let java_flag = java_cancel.clone();

        let java_task = cx.background_executor().spawn(async move {
            sealy_core::java::ensure(&java_dir, &component, major, &java_worker, &java_flag)
        });

        loop {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(200))
                .await;

            let snapshot = java_progress
                .lock()
                .map(|progress| progress.clone())
                .unwrap_or_default();
            let finished = snapshot.finished;
            let ratio = if snapshot.total == 0 {
                0.0
            } else {
                (snapshot.done as f32 / snapshot.total as f32).clamp(0.0, 1.0)
            };
            let detail = if snapshot.total == 0 {
                tr(lang, "正在检测本机 Java…", "Looking for a local Java…").to_owned()
            } else {
                format!("{} / {}", snapshot.done, snapshot.total)
            };

            let _ = this.update(cx, |shell, cx| {
                shell.state.update_message(java_message, |row| {
                    row.progress = Some(ratio);
                    row.detail = detail.clone();
                });
                cx.notify();
            });

            if finished {
                break;
            }
        }

        match java_task.await {
            Ok(java) => {
                let _ = this.update(cx, |shell, cx| {
                    shell.state.install = None;
                    shell.state.java.push(java.clone());
                    shell.state.update_message(java_message, |row| {
                        row.tone = MessageTone::Done;
                        row.progress = Some(1.0);
                        row.detail = format!("Java {} · {}", java.version, java.path.display());
                    });
                    cx.notify();
                });
            }
            Err(err) => {
                let _ = this.update(cx, |shell, cx| {
                    shell.state.install = None;
                    shell.state.update_message(java_message, |row| {
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
