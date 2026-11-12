//! Install flows: a version, or a loader plus its base version.
//!
//! Every phase (version files → assets → Java) reports into a single message
//! row so the message center shows one task, and that row's cancel button
//! aborts the whole flow.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sealy_core::config::SourcePolicy;
use sealy_core::loader::LoaderKind;

use crate::app::SealyShell;
use crate::i18n::tr;
use crate::kit::Context;
use crate::state::{InstallTask, MessageTone, ShellState};

/// One thing the flow has to install.
enum Job {
    /// A version that already has a document on disk (vanilla or loader).
    Version(String),
    /// A loader profile for `game`, written before its libraries are fetched.
    Loader { game: String, kind: LoaderKind },
}

/// Settings shared by every phase of one flow.
#[derive(Clone, Copy)]
struct Opts {
    policy: SourcePolicy,
    concurrency: usize,
    auto_java: bool,
}

/// Installs `id`, reporting progress into the message center.
pub(crate) fn start(state: &mut ShellState, id: String, cx: &mut Context<SealyShell>) {
    spawn(state, cx, format!("安装 {id}"), vec![Job::Version(id)]);
}

/// Installs `kind` for `game`, then fetches the loader libraries.
pub(crate) fn start_loader(
    state: &mut ShellState,
    game: String,
    kind: LoaderKind,
    cx: &mut Context<SealyShell>,
) {
    let title = format!("安装 {} · {game}", kind.label());
    spawn(state, cx, title, vec![Job::Loader { game, kind }]);
}

fn spawn(state: &mut ShellState, cx: &mut Context<SealyShell>, title: String, jobs: Vec<Job>) {
    if state.install.is_some() {
        state.notify(tr(
            state.language(),
            "已有任务在运行，请等待完成",
            "Another install is already running",
        ));
        return;
    }

    let root = state.paths.game.clone();
    let opts = Opts {
        policy: state.settings.download.metadata_source,
        concurrency: usize::from(state.settings.download.concurrency).max(1),
        auto_java: state.settings.java.auto_download,
    };
    let lang = state.language();

    let message_id = state.push_message(
        title,
        tr(lang, "正在准备…", "Preparing…").to_owned(),
        MessageTone::Running,
    );
    state.message_open = true;

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let handle = cancel.clone();
        state.update_message(message_id, move |row| row.cancel = Some(handle));
    }
    state.install = Some(InstallTask {
        message_id,
        progress: Arc::new(Mutex::new(sealy_core::install::Progress::default())),
        cancel: cancel.clone(),
    });

    cx.spawn(async move |this, cx| {
        for job in jobs {
            let outcome = match job {
                Job::Version(id) => {
                    run_version(&this, cx, message_id, &root, id, cancel.clone(), opts).await
                }
                Job::Loader { game, kind } => {
                    let label = kind.label();
                    let _ = this.update(cx, |shell, cx| {
                        shell.state.update_message(message_id, |row| {
                            row.detail = format!("正在获取 {label} 配置…");
                        });
                        cx.notify();
                    });

                    let install = {
                        let root = root.clone();
                        let game = game.clone();
                        cx.background_executor()
                            .spawn(async move { sealy_core::loader::install(&root, kind, &game) })
                            .await
                    };

                    match install {
                        Ok(loader) => {
                            let title = format!("安装 {} · {}", kind.label(), loader.id);
                            let _ = this.update(cx, |shell, cx| {
                                let detail = format!(
                                    "{} · 正在获取库文件…",
                                    tr(shell.state.language(), "加载器已就绪", "Loader ready")
                                );
                                shell.state.update_message(message_id, |row| {
                                    row.title = title.clone();
                                    row.detail = detail.clone();
                                });
                                cx.notify();
                            });
                            run_version(
                                &this,
                                cx,
                                message_id,
                                &root,
                                loader.id.clone(),
                                cancel.clone(),
                                opts,
                            )
                            .await
                        }
                        Err(err) => Err(err.to_string()),
                    }
                }
            };

            if let Err(err) = outcome {
                if err == CANCELLED {
                    finish(&this, cx, message_id, MessageTone::Failed, "已取消", &root);
                } else {
                    finish(&this, cx, message_id, MessageTone::Failed, &err, &root);
                }
                return;
            }
        }

        finish(&this, cx, message_id, MessageTone::Done, "安装完成", &root);
    })
    .detach();
}

/// Marker for a cancelled flow.
const CANCELLED: &str = "\u{0}cancelled";

async fn run_version(
    this: &gpui_kit::WeakEntity<SealyShell>,
    cx: &mut gpui_kit::AsyncApp,
    message_id: u64,
    root: &Path,
    id: String,
    cancel: Arc<AtomicBool>,
    opts: Opts,
) -> Result<(), String> {
    // Phase one: version jar, libraries, natives.
    let files = Arc::new(Mutex::new(sealy_core::install::Progress::default()));
    let task = cx.background_executor().spawn({
        let progress = files.clone();
        let flag = cancel.clone();
        let root = root.to_path_buf();
        let id = id.clone();
        async move {
            sealy_core::install::install(&root, &id, opts.policy, opts.concurrency, progress, flag)
        }
    });

    poll(this, cx, message_id, "版本文件", &files, cancel.clone()).await?;
    let installed = task.await.map_err(|err| err.to_string())?;

    // Phase two: asset objects referenced by the version.
    if let Some((index_id, index_url)) = installed.assets.clone() {
        let assets = Arc::new(Mutex::new(sealy_core::install::Progress::default()));
        let assets_task =
            cx.background_executor().spawn({
                let progress = assets.clone();
                let flag = cancel.clone();
                let root = root.to_path_buf();
                async move {
                    sealy_core::assets::install(&root, &index_url, &index_id, &progress, &flag)
                }
            });

        poll(this, cx, message_id, "资源文件", &assets, cancel.clone()).await?;
        assets_task
            .await
            .map_err(|err| format!("资源文件下载失败：{err}"))?;
    }

    // Phase three: a matching Java runtime.
    let Some((component, major)) = installed.java.filter(|_| opts.auto_java) else {
        return Ok(());
    };

    let java = Arc::new(Mutex::new(sealy_core::install::Progress::default()));
    let java_task = cx.background_executor().spawn({
        let progress = java.clone();
        let flag = cancel.clone();
        let root = root.to_path_buf();
        async move { sealy_core::java::ensure(&root, &component, major, &progress, &flag) }
    });

    poll(this, cx, message_id, "Java", &java, cancel.clone()).await?;
    let java = java_task
        .await
        .map_err(|err| format!("Java 准备失败：{err}"))?;

    let _ = this.update(cx, |shell, cx| {
        shell.state.java.push(java);
        cx.notify();
    });
    Ok(())
}

/// Mirrors a progress handle into the message until the phase reports finished.
async fn poll(
    this: &gpui_kit::WeakEntity<SealyShell>,
    cx: &mut gpui_kit::AsyncApp,
    message_id: u64,
    label: &str,
    progress: &Arc<Mutex<sealy_core::install::Progress>>,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    loop {
        cx.background_executor()
            .timer(Duration::from_millis(200))
            .await;

        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.to_owned());
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
            return Ok(());
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

fn finish(
    this: &gpui_kit::WeakEntity<SealyShell>,
    cx: &mut gpui_kit::AsyncApp,
    id: u64,
    tone: MessageTone,
    detail: &str,
    root: &Path,
) {
    // A loader install adds a version directory; refresh the picker data.
    let installed = sealy_core::version::installed(root);
    let detail = detail.to_owned();
    let _ = this.update(cx, |shell, cx| {
        shell.state.install = None;
        shell.state.installed = installed;
        shell.state.update_message(id, |row| {
            row.tone = tone;
            row.progress = if tone == MessageTone::Done {
                Some(1.0)
            } else {
                None
            };
            row.cancel = None;
            row.detail = detail;
        });
        cx.notify();
    });
}
