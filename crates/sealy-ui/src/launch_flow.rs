//! Shared launch flow: used by the home action bar and the instances page.

use std::path::PathBuf;

use sealy_core::instances::Instance;
use sealy_core::java::JavaInstall;

use crate::app::SealyShell;
use crate::i18n::tr;
use crate::kit::Context;
use crate::state::{MessageTone, ShellState};

/// Launches the selected instance, reporting into the message center.
pub(crate) fn start_selected(state: &mut ShellState, cx: &mut Context<SealyShell>) {
    let lang = state.language();
    let Some(instance) = state.instances.current().cloned() else {
        state.notify(tr(
            lang,
            "还没有实例，请先在实例页新建",
            "No instance yet — create one on the instances page",
        ));
        return;
    };
    start(state, instance, cx);
}

/// Launches `instance`.
pub(crate) fn start(state: &mut ShellState, instance: Instance, cx: &mut Context<SealyShell>) {
    let lang = state.language();
    let message_id = state.push_message(
        format!("启动 {}", instance.name),
        tr(lang, "正在准备启动…", "Preparing to launch…").to_owned(),
        MessageTone::Running,
    );
    state.message_open = true;

    let root = state.paths.game.clone();
    let instance_dir = sealy_core::instances::directory(&root, &instance.name);
    let _ = std::fs::create_dir_all(&instance_dir);

    let java_list = state.java.clone();
    let memory = state.settings.java.memory.max_mb;
    let width = state.settings.launch.width;
    let height = state.settings.launch.height;
    let account = state
        .accounts
        .current()
        .cloned()
        .unwrap_or_else(|| sealy_core::accounts::Account::offline("Player"));
    let client_id = state.settings.auth.microsoft_client_id.clone();
    let tokens = state.tokens.clone();
    let instance_name = instance.name.clone();
    let version_id = instance.version_id.clone();

    cx.spawn(async move |this, cx| {
        let outcome = cx.background_executor().spawn(async move {
            launch_blocking(
                root,
                instance_dir,
                version_id,
                java_list,
                memory,
                width,
                height,
                account,
                client_id,
                tokens,
            )
        });

        match outcome.await {
            Ok((status, tokens)) => {
                let _ = this.update(cx, |shell, cx| {
                    shell.state.tokens = tokens;
                    shell.state.save_tokens();
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Done;
                        row.progress = None;
                        row.detail = status;
                    });
                    shell.state.instances.touch(&instance_name, now_epoch());
                    shell.state.save_instances();
                    cx.notify();
                });
            }
            Err(err) => {
                let _ = this.update(cx, |shell, cx| {
                    shell.state.update_message(message_id, |row| {
                        row.tone = MessageTone::Failed;
                        row.progress = None;
                        row.detail = err;
                    });
                    cx.notify();
                });
            }
        }
    })
    .detach();
}

/// Resolves Java and credentials, builds the command and waits for the game.
#[allow(clippy::too_many_arguments)]
fn launch_blocking(
    root: PathBuf,
    instance_dir: PathBuf,
    version_id: String,
    java_list: Vec<JavaInstall>,
    memory: u32,
    width: u32,
    height: u32,
    account: sealy_core::accounts::Account,
    client_id: String,
    mut tokens: sealy_core::auth::TokenStore,
) -> Result<(String, sealy_core::auth::TokenStore), String> {
    let document = root
        .join("versions")
        .join(&version_id)
        .join(format!("{version_id}.json"));
    // Loader profiles inherit their jar, assets and arguments from the parent.
    let doc = sealy_core::version::resolve(&root, &version_id).map_err(|err| err.to_string())?;
    let _ = document;

    let required = doc
        .java_version
        .as_ref()
        .map(|java| java.major_version)
        .unwrap_or(0);
    let java = java_list
        .iter()
        .find(|install| install.satisfies(required))
        .or_else(|| java_list.first())
        .ok_or_else(|| "没有可用的 Java，请先在下载页安装该版本".to_owned())?;

    let access_token = sealy_core::auth::ensure_minecraft_token(&mut tokens, &account, &client_id)
        .map_err(|err| err.to_string())?;
    let (user_type, jvm_extra) = match account.kind {
        sealy_core::accounts::AccountKind::Microsoft => ("msa", Vec::new()),
        sealy_core::accounts::AccountKind::Authlib => {
            let server = account
                .server
                .clone()
                .ok_or_else(|| "外置账号缺少服务器地址，请重新登录".to_owned())?;
            let jar = sealy_core::authlib::ensure_injector(&root).map_err(|err| err.to_string())?;
            (
                "mojang",
                vec![sealy_core::authlib::javaagent(&jar, &server)],
            )
        }
        sealy_core::accounts::AccountKind::Offline => ("legacy", Vec::new()),
    };

    let spec = sealy_core::launch::LaunchSpec {
        root: &root,
        game_dir: &instance_dir,
        java: &java.path,
        version_id: &version_id,
        username: &account.name,
        uuid: &account.uuid,
        access_token: &access_token,
        user_type,
        jvm_extra: &jvm_extra,
        max_memory_mb: memory,
        width,
        height,
    };

    let command = sealy_core::launch::build_command(&doc, &spec).map_err(|err| err.to_string())?;
    sealy_core::launch::extract_natives(&root, &doc, &version_id).map_err(|err| err.to_string())?;

    let log = sealy_core::launch::log_path(&root, &version_id);
    let mut child =
        sealy_core::launch::spawn(&command, &instance_dir, &log).map_err(|err| err.to_string())?;
    let status = child.wait().map_err(|err| err.to_string())?;

    Ok((
        format!("退出码 {} · {}", status.code().unwrap_or(-1), log.display()),
        tokens,
    ))
}

/// Seconds since the Unix epoch.
pub(crate) fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|delta| delta.as_secs())
        .unwrap_or(0)
}
