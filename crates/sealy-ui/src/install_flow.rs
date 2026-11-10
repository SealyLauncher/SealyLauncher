//! Install flow: version files, assets, then a matching Java runtime.
//!
//! Every phase reports into one message row so the message center shows a
//! single task, and the row's cancel button aborts the whole install.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let retry = id.clone();
        let cancel_handle = cancel.clone();
        state.update_message(message_id, move |row| {
            row.retry = Some(retry);
            row.cancel = Some(cancel_handle);
        });
    }
    state.install = Some(InstallTask {
        message_id,
        progress: Arc::new(Mutex::new(sealy_core::install::Progress::default())),
        cancel: cancel.clone(),
    });

    cx.spawn(async move |this, cx| {
        // Phase one: version jar, libraries, natives.
        let files_progress = Arc::new(Mutex::new(sealy_core::install::Progress::default()));
        let task = cx.background_executor().spawn({
            let progress = files_progress.clone();
            let flag = cancel.clone();
            let root = game.clone();
            let task_id = id.clone();
            async move {
                sealy_core::install::install(&root, &task_id, policy, concurrency, progress, flag)
            }
        });

        if !poll(
            &this,
            cx,
            message_id,
            "版本文件",
            &files_progress,
            cancel.clone(),
        )
        .await
        {
            finish_cancelled(&this, cx, message_id);
            return;
        }

        let installed = match task.await {
            Ok(installed) => installed,
            Err(err) => {
                fail(&this, cx, message_id, err.to_string());
                return;
            }
        };

        // Phase two: asset objects referenced by the version.
        if let Some((index_id, index_url)) = installed.assets.clone() {
            let assets_progress = Arc::new(Mutex::new(sealy_core::install::Progress::default()));
            let assets_task = cx.background_executor().spawn({
                let progress = assets_progress.clone();
                let flag = cancel.clone();
                let root = game.clone();
                async move {
                    sealy_core::assets::install(&root, &index_url, &index_id, &progress, &flag)
                }
            });

            if !poll(
                &this,
                cx,
                message_id,
                "资源文件",
                &assets_progress,
                cancel.clone(),
            )
            .await
            {
                finish_cancelled(&this, cx, message_id);
                return;
            }

            if let Err(err) = assets_task.await {
                fail(&this, cx, message_id, format!("资源文件下载失败：{err}"));
                return;
            }
        }

        let summary = if installed.bytes == 0 {
            format!("{} 个文件 · 已是最新", installed.files)
        } else {
            format!(
                "{} 个文件 · {} MB",
                installed.files,
                installed.bytes / 1_048_576
            )
        };
        {
            let text = summary.clone();
            let retry = id.clone();
            let _ = this.update(cx, |shell, cx| {
                shell.state.update_message(message_id, move |row| {
                    row.progress = Some(1.0);
                    row.detail = text;
                    row.retry = Some(retry);
                });
                cx.notify();
            });
        }

        // Phase three: a matching Java runtime.
        let Some((component, major)) = installed.java.filter(|_| auto_java) else {
            let _ = this.update(cx, |shell, cx| {
                shell.state.install = None;
                shell.state.update_message(message_id, |row| {
                    row.tone = MessageTone::Done;
                    row.cancel = None;
                    row.detail = summary.clone();
                });
                cx.notify();
            });
            return;
        };

        let java_progress = Arc::new(Mutex::new(sealy_core::install::Progress::default()));
        let java_task = cx.background_executor().spawn({
            let progress = java_progress.clone();
            let flag = cancel.clone();
            let root = game.clone();
            async move { sealy_core::java::ensure(&root, &component, major, &progress, &flag) }
        });

        if !poll(
            &this,
            cx,
            message_id,
            "Java",
            &java_progress,
            cancel.clone(),
        )
        .await
        {
            finish_cancelled(&this, cx, message_id);
            return;
        }

        let java = match java_task.await {
            Ok(java) => java,
            Err(err) => {
                fail(&this, cx, message_id, format!("Java 准备失败：{err}"));
                return;
            }
        };

        let _ = this.update(cx, |shell, cx| {
            shell.state.install = None;
            shell.state.java.push(java.clone());
            shell.state.update_message(message_id, |row| {
                row.tone = MessageTone::Done;
                row.progress = Some(1.0);
                row.cancel = None;
                row.detail = format!("Java {} 已就绪", java.version);
            });
            cx.notify();
        });
    })
    .detach();
}

/// Mirrors a progress handle into the message until the phase reports finished.
///
/// Returns `false` as soon as the cancel flag is set.
async fn poll(
    this: &gpui_kit::WeakEntity<SealyShell>,
    cx: &mut gpui_kit::AsyncApp,
    message_id: u64,
    label: &str,
    progress: &Arc<Mutex<sealy_core::install::Progress>>,
    cancel: Arc<AtomicBool>,
) -> bool {
    loop {
        cx.background_executor()
            .timer(Duration::from_millis(200))
            .await;

        if cancel.load(Ordering::Relaxed) {
            return false;
        }

        let snapshot = progress
            .lock()
            .map(|progress| progress.clone())
            .unwrap_or_default();
        let finished = snapshot.finished;

        let _ = this.update(cx, |shell, cx| {
            let detail = format!(
                "{label} · {}",
                phase_detail(&snapshot, shell.state.language())
            );
            let ratio = if snapshot.total == 0 {
                0.0
            } else {
                (snapshot.done as f32 / snapshot.total as f32).clamp(0.0, 1.0)
            };
            shell.state.update_message(message_id, |row| {
                row.progress = Some(ratio);
                row.detail = detail;
            });
            cx.notify();
        });

        if finished {
            return true;
        }
    }
}

fn phase_detail(
    snapshot: &sealy_core::install::Progress,
    lang: sealy_core::config::Language,
) -> String {
    if snapshot.total == 0 {
        return tr(lang, "正在准备…", "Preparing…").to_owned();
    }

    let name = snapshot
        .current
        .rsplit('/')
        .next()
        .unwrap_or(&snapshot.current);
    match (snapshot.current_bytes * 100).checked_div(snapshot.current_size) {
        Some(percent) => format!("{name} · {percent}% · {}/{}", snapshot.done, snapshot.total),
        None => format!("{} / {} · {name}", snapshot.done, snapshot.total),
    }
}

fn fail(
    this: &gpui_kit::WeakEntity<SealyShell>,
    cx: &mut gpui_kit::AsyncApp,
    id: u64,
    detail: String,
) {
    let _ = this.update(cx, |shell, cx| {
        shell.state.install = None;
        shell.state.update_message(id, |row| {
            row.tone = MessageTone::Failed;
            row.progress = None;
            row.cancel = None;
            row.detail = detail;
        });
        cx.notify();
    });
}

fn finish_cancelled(this: &gpui_kit::WeakEntity<SealyShell>, cx: &mut gpui_kit::AsyncApp, id: u64) {
    let _ = this.update(cx, |shell, cx| {
        shell.state.install = None;
        shell.state.update_message(id, |row| {
            row.tone = MessageTone::Failed;
            row.progress = None;
            row.cancel = None;
            row.detail = "已取消".to_owned();
        });
        cx.notify();
    });
}
