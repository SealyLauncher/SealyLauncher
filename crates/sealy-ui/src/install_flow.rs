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
    /// A loader install for `game`, optionally tied to a new instance.
    Loader(LoaderRequest),
    /// A modpack archive whose instance still has to be created.
    Modpack(std::path::PathBuf, sealy_core::modpack::ModpackPlan),
}

/// Everything needed to install a loader.
#[derive(Clone)]
pub(crate) struct LoaderRequest {
    /// Minecraft version the loader extends.
    pub game: String,
    /// Loader to install.
    pub kind: LoaderKind,
    /// Specific loader build; `None` picks the newest stable one.
    pub version: Option<String>,
    /// Local installer jar, required for OptiFine.
    pub jar: Option<std::path::PathBuf>,
    /// Instance to create once the loader is installed.
    pub instance_name: Option<String>,
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

/// Opens the native picker and imports the chosen modpack archive.
pub(crate) fn prompt_modpack(cx: &mut crate::kit::Context<crate::app::SealyShell>) {
    let receiver = cx.prompt_for_paths(crate::kit::PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("选择整合包（.mrpack / .zip）".into()),
    });
    cx.spawn(async move |this, cx| {
        let Ok(Ok(Some(paths))) = receiver.await else {
            return;
        };
        let Some(path) = paths.into_iter().next() else {
            return;
        };
        let _ = this.update(cx, |shell, cx| {
            start_modpack(&mut shell.state, path, cx);
            cx.notify();
        });
    })
    .detach();
}
/// Imports a modpack archive as a new instance.
pub(crate) fn start_modpack(
    state: &mut ShellState,
    archive: std::path::PathBuf,
    cx: &mut Context<SealyShell>,
) {
    let plan = match sealy_core::modpack::inspect(&archive) {
        Ok(plan) => plan,
        Err(err) => {
            state.notify(err.to_string());
            return;
        }
    };
    let title = format!("导入整合包 · {}", plan.name);
    spawn(state, cx, title, vec![Job::Modpack(archive, plan)]);
}

/// Runs one loader install and, when asked, creates its instance afterwards.
pub(crate) fn start_loader_request(
    state: &mut ShellState,
    request: LoaderRequest,
    cx: &mut Context<SealyShell>,
) {
    let title = match &request.instance_name {
        Some(name) => format!("安装 {} · {name}", request.kind.label()),
        None => format!("安装 {} · {}", request.kind.label(), request.game),
    };
    spawn(state, cx, title, vec![Job::Loader(request)]);
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
    // Installer jars have to run on a real JVM; prefer a detected runtime.
    let java = state
        .java
        .first()
        .map(|install| install.path.clone())
        .or_else(|| {
            sealy_core::java::runtime_java_candidates(&root)
                .into_iter()
                .next()
        });

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
        // Instance created by this flow, once every phase succeeded.
        let mut created: Option<(String, String)> = None;
        // Modpack flows create their instance themselves and only need a
        // different completion label.
        let mut modpack_imported = false;

        for job in jobs {
            let outcome = match job {
                Job::Version(id) => {
                    run_version(&this, cx, message_id, &root, id, cancel.clone(), opts).await
                }
                Job::Loader(request) => {
                    let label = request.kind.label();
                    let _ = this.update(cx, |shell, cx| {
                        shell.state.update_message(message_id, |row| {
                            row.detail = format!("正在获取 {label} 安装文件…");
                        });
                        cx.notify();
                    });

                    let install = {
                        let root = root.clone();
                        let request = request.clone();
                        let java = java.clone();
                        cx.background_executor()
                            .spawn(async move { run_loader(&root, &request, java.as_deref()) })
                            .await
                    };

                    match install {
                        Ok(loader) => {
                            let loader_id = loader.id.clone();
                            let title = format!("安装 {} · {}", request.kind.label(), loader_id);
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
                            let result = run_version(
                                &this,
                                cx,
                                message_id,
                                &root,
                                loader_id.clone(),
                                cancel.clone(),
                                opts,
                            )
                            .await;
                            if result.is_ok()
                                && let Some(name) = request.instance_name.clone()
                            {
                                created = Some((name, loader_id));
                            }
                            result
                        }
                        Err(err) => Err(err.to_string()),
                    }
                }
                Job::Modpack(archive, plan) => {
                    modpack_imported = true;
                    run_modpack(
                        &this,
                        cx,
                        message_id,
                        &root,
                        archive,
                        plan,
                        cancel.clone(),
                        opts,
                    )
                    .await
                }
            };

            if let Err(err) = outcome {
                if err == CANCELLED {
                    finish(
                        &this,
                        cx,
                        message_id,
                        MessageTone::Failed,
                        "已取消",
                        &root,
                        None,
                    );
                } else {
                    finish(
                        &this,
                        cx,
                        message_id,
                        MessageTone::Failed,
                        &err,
                        &root,
                        None,
                    );
                }
                return;
            }
        }

        finish(
            &this,
            cx,
            message_id,
            MessageTone::Done,
            if modpack_imported {
                "整合包已导入"
            } else {
                "安装完成"
            },
            &root,
            created,
        );
    })
    .detach();
}

/// Installs the loader itself (profile document or official installer run).
fn run_loader(
    root: &Path,
    request: &LoaderRequest,
    java: Option<&Path>,
) -> Result<sealy_core::loader::LoaderInstall, sealy_core::error::CoreError> {
    match &request.jar {
        Some(jar) => {
            let java = java.ok_or_else(|| {
                sealy_core::error::CoreError::Invalid(
                    "安装加载器需要 Java，请先在设置里添加或自动下载一个 Java".to_owned(),
                )
            })?;
            sealy_core::loader::install_from_jar(root, &request.game, request.kind, jar, java)
        }
        None => sealy_core::loader::install(
            root,
            request.kind,
            &request.game,
            request.version.as_deref(),
            java,
        ),
    }
}
/// Installs the pack's loader (if any), creates its instance and downloads the
/// pack files into it.
#[allow(clippy::too_many_arguments)]
async fn run_modpack(
    this: &gpui_kit::WeakEntity<SealyShell>,
    cx: &mut gpui_kit::AsyncApp,
    message_id: u64,
    root: &Path,
    archive: std::path::PathBuf,
    plan: sealy_core::modpack::ModpackPlan,
    cancel: Arc<AtomicBool>,
    opts: Opts,
) -> Result<(), String> {
    // Phase one: the version the pack runs on.
    let _ = this.update(cx, |shell, cx| {
        shell.state.update_message(message_id, |row| {
            row.detail = format!("准备 {} · {}", plan.game_version, plan.format.label());
        });
        cx.notify();
    });

    let version_id = match &plan.loader {
        Some(spec) => {
            let java = {
                let mut found = None;
                let _ = this.update(cx, |shell, _| {
                    found = shell
                        .state
                        .java
                        .first()
                        .map(|install| install.path.clone())
                        .or_else(|| {
                            sealy_core::java::runtime_java_candidates(&shell.state.paths.game)
                                .into_iter()
                                .next()
                        });
                });
                found
            };
            let spec = spec.clone();
            let game = plan.game_version.clone();
            let root_buf = root.to_path_buf();
            let loader = cx
                .background_executor()
                .spawn(async move {
                    sealy_core::loader::install(
                        &root_buf,
                        spec.kind,
                        &game,
                        spec.version.as_deref(),
                        java.as_deref(),
                    )
                })
                .await
                .map_err(|err| err.to_string())?;

            let loader_id = loader.id.clone();
            let _ = this.update(cx, |shell, cx| {
                shell.state.update_message(message_id, |row| {
                    row.detail = format!("加载器已就绪 · {loader_id}");
                });
                cx.notify();
            });
            run_version(
                this,
                cx,
                message_id,
                root,
                loader_id.clone(),
                cancel.clone(),
                opts,
            )
            .await?;
            loader_id
        }
        None => {
            let game = plan.game_version.clone();
            run_version(
                this,
                cx,
                message_id,
                root,
                game.clone(),
                cancel.clone(),
                opts,
            )
            .await?;
            game
        }
    };

    // Phase two: create the instance the pack lives in.
    let mut instance_name = String::new();
    let _ = this.update(cx, |shell, cx| {
        let instance = shell.state.instances.create_named(&plan.name, &version_id);
        instance_name = instance.name.clone();
        shell.state.save_instances();
        let directory = sealy_core::instances::directory(&shell.state.paths.game, &instance_name);
        let _ = std::fs::create_dir_all(&directory);
        shell.state.instance_section = crate::state::InstanceSection::Overview;
        shell.state.route = crate::state::Route::Instance;
        cx.notify();
    });
    if instance_name.is_empty() {
        return Err("无法创建整合包实例".to_owned());
    }

    // Phase three: files and overrides.
    let instance_dir = sealy_core::instances::directory(root, &instance_name);
    let progress = Arc::new(Mutex::new(sealy_core::install::Progress::default()));
    let task = cx.background_executor().spawn({
        let progress = progress.clone();
        let flag = cancel.clone();
        let archive = archive.clone();
        let plan = plan.clone();
        let instance_dir = instance_dir.clone();
        async move {
            sealy_core::modpack::import_files(&instance_dir, &archive, &plan, &progress, &flag)
        }
    });

    poll(
        this,
        cx,
        message_id,
        "整合包文件",
        &progress,
        cancel.clone(),
    )
    .await?;
    let summary = task.await.map_err(|err| err.to_string())?;

    let title = format!("整合包 · {}", plan.name);
    let detail = if summary.skipped > 0 {
        format!(
            "已导入 {} 个文件、{} 个覆盖文件；{} 个条目无法自动下载，请手动处理",
            summary.files, summary.overrides, summary.skipped
        )
    } else {
        format!(
            "已导入 {} 个文件、{} 个覆盖文件",
            summary.files, summary.overrides
        )
    };
    let _ = this.update(cx, |shell, cx| {
        shell.state.update_message(message_id, |row| {
            row.title = title;
            row.detail = detail;
        });
        cx.notify();
    });
    Ok(())
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
    created: Option<(String, String)>,
) {
    // A loader install adds a version directory; refresh the picker data.
    let installed = sealy_core::version::installed(root);
    let mut detail = detail.to_owned();
    let _ = this.update(cx, |shell, cx| {
        shell.state.install = None;
        shell.state.installed = installed;
        if let Some((_name, version_id)) = created {
            let instance = shell.state.instances.create(&version_id);
            let name = instance.name.clone();
            shell.state.instances.selected = Some(name.clone());
            shell.state.save_instances();
            let dir = sealy_core::instances::directory(&shell.state.paths.game, &name);
            let _ = std::fs::create_dir_all(&dir);
            detail = format!("已创建实例 {name} · {version_id}");
        }
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
