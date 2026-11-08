//! Turning a version document into a runnable command line.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde::Deserialize;

use crate::error::CoreError;
use crate::version::{self, Library, OsRule, RuleAction, VersionDocument};

/// Everything the command needs that is not in the version document.
#[derive(Debug, Clone)]
pub struct LaunchSpec<'a> {
    /// Shared repository (`.minecraft`): versions, libraries, assets.
    pub root: &'a Path,
    /// Instance game directory that receives saves and options.
    pub game_dir: &'a Path,
    /// Java executable.
    pub java: &'a Path,
    /// Version id being launched.
    pub version_id: &'a str,
    /// Player name.
    pub username: &'a str,
    /// Player UUID (offline UUID when signed out).
    pub uuid: &'a str,
    /// Maximum heap in MB.
    pub max_memory_mb: u32,
    /// Window width.
    pub width: u32,
    /// Window height.
    pub height: u32,
}

/// Builds the full command line for a version.
pub fn build_command(
    doc: &VersionDocument,
    spec: &LaunchSpec<'_>,
) -> Result<Vec<String>, CoreError> {
    let natives = spec
        .game_dir
        .join("versions")
        .join(spec.version_id)
        .join("natives");
    let classpath = classpath(spec.root, doc, spec.version_id);
    let cp_separator = if cfg!(windows) { ";" } else { ":" };

    let mut placeholders: BTreeMap<&str, String> = BTreeMap::new();
    placeholders.insert("auth_player_name", spec.username.to_owned());
    placeholders.insert("version_name", spec.version_id.to_owned());
    placeholders.insert("game_directory", spec.game_dir.display().to_string());
    placeholders.insert(
        "assets_root",
        spec.root.join("assets").display().to_string(),
    );
    placeholders.insert(
        "assets_index_name",
        doc.asset_index
            .as_ref()
            .map(|index| index.id.clone())
            .unwrap_or_else(|| spec.version_id.to_owned()),
    );
    placeholders.insert("auth_uuid", spec.uuid.to_owned());
    placeholders.insert("auth_access_token", "0".to_owned());
    placeholders.insert("clientid", "0".to_owned());
    placeholders.insert("auth_xuid", "0".to_owned());
    placeholders.insert("user_type", "legacy".to_owned());
    placeholders.insert("version_type", "release".to_owned());
    placeholders.insert("natives_directory", natives.display().to_string());
    placeholders.insert("launcher_name", "SealyLauncher".to_owned());
    placeholders.insert("launcher_version", crate::version().to_owned());
    placeholders.insert("classpath", classpath.join(cp_separator));
    placeholders.insert("classpath_separator", cp_separator.to_owned());
    placeholders.insert(
        "library_directory",
        spec.root.join("libraries").display().to_string(),
    );
    placeholders.insert("resolution_width", spec.width.to_string());
    placeholders.insert("resolution_height", spec.height.to_string());

    let mut command = vec![spec.java.display().to_string()];
    command.push(format!("-Xmx{}M", spec.max_memory_mb));
    command.push(format!("-Xms{}M", (spec.max_memory_mb / 2).max(512)));

    let jvm = jvm_args(doc);
    if jvm.is_empty() {
        command.push(format!("-Djava.library.path={}", natives.display()));
        command.push("-cp".to_owned());
        command.push(classpath.join(cp_separator));
    } else {
        for token in jvm {
            command.push(substitute(&token, &placeholders));
        }
    }

    command.push(doc.main_class.clone());

    let game = game_args(doc);
    if game.is_empty() {
        command.extend(legacy_game_args(doc, spec, &placeholders));
    } else {
        for token in game {
            command.push(substitute(&token, &placeholders));
        }
    }

    Ok(command)
}

fn substitute(token: &str, values: &BTreeMap<&str, String>) -> String {
    let mut output = token.to_owned();
    for (key, value) in values {
        output = output.replace(&format!("${{{key}}}"), value);
    }
    output
}

/// Absolute classpath entries, libraries first.
pub fn classpath(game_dir: &Path, doc: &VersionDocument, version_id: &str) -> Vec<String> {
    let mut entries = Vec::new();
    for library in &doc.libraries {
        if !library.allowed() {
            continue;
        }
        if let Some(path) = version::maven_path(&library.name) {
            entries.push(game_dir.join("libraries").join(path).display().to_string());
        }
    }
    entries.push(
        game_dir
            .join("versions")
            .join(version_id)
            .join(format!("{version_id}.jar"))
            .display()
            .to_string(),
    );
    entries
}

/// Natives jars that must be unpacked before launch.
pub fn native_jars(doc: &VersionDocument) -> Vec<&Library> {
    doc.libraries
        .iter()
        .filter(|library| library.allowed() && library.name.contains(":natives-"))
        .collect()
}

/// Extracts native libraries into `versions/<id>/natives`.
pub fn extract_natives(
    game_dir: &Path,
    doc: &VersionDocument,
    version_id: &str,
) -> Result<usize, CoreError> {
    let destination = game_dir.join("versions").join(version_id).join("natives");
    fs::create_dir_all(&destination)?;

    let mut extracted = 0;
    for library in native_jars(doc) {
        let Some(path) = version::maven_path(&library.name) else {
            continue;
        };
        let jar = game_dir.join("libraries").join(path);
        if !jar.is_file() {
            continue;
        }
        extracted += extract_archive(&jar, &destination)?;
    }

    Ok(extracted)
}

fn extract_archive(jar: &Path, destination: &Path) -> Result<usize, CoreError> {
    let file = fs::File::open(jar)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|err| CoreError::Io(io::Error::other(format!("{}: {err}", jar.display()))))?;

    let mut count = 0;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| CoreError::Io(io::Error::other(err.to_string())))?;
        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_owned();
        if !(name.ends_with(".dll") || name.ends_with(".so") || name.ends_with(".dylib")) {
            continue;
        }
        let Some(file_name) = Path::new(&name).file_name() else {
            continue;
        };

        let target = destination.join(file_name);
        let mut output = fs::File::create(&target)?;
        io::copy(&mut entry, &mut output)?;
        count += 1;
    }

    Ok(count)
}

/// Raw JVM argument tokens after rule filtering.
fn jvm_args(doc: &VersionDocument) -> Vec<String> {
    doc.arguments
        .as_ref()
        .and_then(|arguments| arguments.get("jvm"))
        .map(tokens)
        .unwrap_or_default()
}

/// Raw game argument tokens after rule filtering.
fn game_args(doc: &VersionDocument) -> Vec<String> {
    doc.arguments
        .as_ref()
        .and_then(|arguments| arguments.get("game"))
        .map(tokens)
        .unwrap_or_default()
}

/// Argument entry with platform rules.
#[derive(Debug, Deserialize)]
struct RawArgument {
    #[serde(default)]
    rules: Vec<RawRule>,
    value: serde_json::Value,
}

/// Rule attached to an argument.
#[derive(Debug, Deserialize)]
struct RawRule {
    action: RuleAction,
    #[serde(default)]
    os: Option<OsRule>,
    #[serde(default)]
    features: Option<BTreeMap<String, bool>>,
}

impl RawRule {
    fn applies(&self) -> bool {
        if self.features.is_some() {
            // Demo mode and custom resolution are never requested.
            return false;
        }
        match &self.os {
            None => true,
            Some(os) => {
                let name_ok = os
                    .name
                    .as_deref()
                    .is_none_or(|name| name == version::current_os());
                let arch_ok = os.arch.as_deref().is_none_or(|arch| match arch {
                    "x86" => cfg!(target_arch = "x86"),
                    "x86_64" => cfg!(target_arch = "x86_64"),
                    "arm64" => cfg!(target_arch = "aarch64"),
                    _ => true,
                });
                name_ok && arch_ok
            }
        }
    }
}

fn allowed(rules: &[RawRule]) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allow = false;
    for rule in rules {
        if rule.applies() {
            allow = rule.action == RuleAction::Allow;
        }
    }
    allow
}

fn tokens(value: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    let Some(items) = value.as_array() else {
        return out;
    };

    for item in items {
        match item {
            serde_json::Value::String(text) => out.push(text.clone()),
            serde_json::Value::Object(_) => {
                let Ok(argument) = serde_json::from_value::<RawArgument>(item.clone()) else {
                    continue;
                };
                if !allowed(&argument.rules) {
                    continue;
                }
                match &argument.value {
                    serde_json::Value::String(text) => out.push(text.clone()),
                    serde_json::Value::Array(items) => {
                        out.extend(
                            items
                                .iter()
                                .filter_map(|item| item.as_str().map(str::to_owned)),
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    out
}

/// Legacy `minecraftArguments` fallback for old versions.
fn legacy_game_args(
    doc: &VersionDocument,
    spec: &LaunchSpec<'_>,
    placeholders: &BTreeMap<&str, String>,
) -> Vec<String> {
    let mut out = vec![
        "--username".to_owned(),
        spec.username.to_owned(),
        "--version".to_owned(),
        spec.version_id.to_owned(),
        "--gameDir".to_owned(),
        spec.game_dir.display().to_string(),
        "--assetsDir".to_owned(),
        spec.game_dir.join("assets").display().to_string(),
        "--assetIndex".to_owned(),
        placeholders
            .get("assets_index_name")
            .cloned()
            .unwrap_or_default(),
        "--uuid".to_owned(),
        spec.uuid.to_owned(),
        "--accessToken".to_owned(),
        "0".to_owned(),
        "--userType".to_owned(),
        "legacy".to_owned(),
        "--versionType".to_owned(),
        "release".to_owned(),
    ];
    let _ = doc;
    out.shrink_to_fit();
    out
}

/// Starts the game and returns the child process.
pub fn spawn(command: &[String], game_dir: &Path, log: &Path) -> Result<Child, CoreError> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| CoreError::Network("empty command".to_owned()))?;

    if let Some(parent) = log.parent() {
        fs::create_dir_all(parent)?;
    }
    let stdout = fs::File::create(log)?;
    let stderr = stdout.try_clone()?;

    Command::new(program)
        .args(args)
        .current_dir(game_dir)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(CoreError::Io)
}

/// Path used for the launch log of a version.
pub fn log_path(game_dir: &Path, version_id: &str) -> PathBuf {
    game_dir
        .join("logs")
        .join(format!("{version_id}-latest.log"))
}
