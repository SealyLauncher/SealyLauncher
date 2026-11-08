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
    let username = "Haikuorenyuyue".to_owned();
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
                username,
            )
        });

        match outcome.await {
            Ok(status) => {
                let _ = this.update(cx, |shell, cx| {
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

/// Resolves Java, builds the command and waits for the game to exit.
#[allow(clippy::too_many_arguments)]
fn launch_blocking(
    root: PathBuf,
    instance_dir: PathBuf,
    version_id: String,
    java_list: Vec<JavaInstall>,
    memory: u32,
    width: u32,
    height: u32,
    username: String,
) -> Result<String, String> {
    let document = root
        .join("versions")
        .join(&version_id)
        .join(format!("{version_id}.json"));
    let doc = sealy_core::version::load(&document).map_err(|err| err.to_string())?;

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

    let uuid = offline_uuid(&username);
    let spec = sealy_core::launch::LaunchSpec {
        root: &root,
        game_dir: &instance_dir,
        java: &java.path,
        version_id: &version_id,
        username: &username,
        uuid: &uuid,
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

    Ok(format!(
        "退出码 {} · {}",
        status.code().unwrap_or(-1),
        log.display()
    ))
}

/// Deterministic offline UUID so worlds keep the same player identity.
pub(crate) fn offline_uuid(name: &str) -> String {
    use sha1::{Digest, Sha1};

    let mut hasher = Sha1::new();
    hasher.update(format!("OfflinePlayer:{name}").as_bytes());
    let digest = hasher.finalize();

    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Seconds since the Unix epoch.
pub(crate) fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|delta| delta.as_secs())
        .unwrap_or(0)
}
