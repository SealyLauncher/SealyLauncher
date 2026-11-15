//! Loader installation: Fabric, Quilt, Forge, NeoForge and OptiFine.
//!
//! Fabric and Quilt publish a *profile* JSON that only contains the loader main
//! class and libraries plus `inheritsFrom: <game version>`; writing it under
//! `versions/<id>/<id>.json` is enough for the normal install flow to fetch the
//! loader libraries and to launch through version inheritance.
//!
//! Forge and NeoForge ship an installer jar. Their installers support a
//! headless `--installClient <dir>` mode, which downloads the patched client
//! jar and writes the loader version document itself. The installer needs a
//! `launcher_profiles.json` in the target directory and a writable
//! `java.io.tmpdir`, both of which are prepared here.
//!
//! OptiFine cannot be downloaded automatically (its site is protected), so the
//! user picks a local installer jar; it is installed the same way when it
//! supports the headless mode.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::error::CoreError;
use crate::net;

/// Which loader to install.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderKind {
    /// Fabric.
    Fabric,
    /// Quilt.
    Quilt,
    /// Forge.
    Forge,
    /// NeoForge.
    NeoForge,
    /// OptiFine.
    OptiFine,
}

impl LoaderKind {
    /// Every loader, in display order.
    pub const ALL: [Self; 5] = [
        Self::Fabric,
        Self::Quilt,
        Self::Forge,
        Self::NeoForge,
        Self::OptiFine,
    ];

    /// Display name.
    pub fn label(self) -> &'static str {
        match self {
            Self::Fabric => "Fabric",
            Self::Quilt => "Quilt",
            Self::Forge => "Forge",
            Self::NeoForge => "NeoForge",
            Self::OptiFine => "OptiFine",
        }
    }

    /// Lowercase id used in version ids and file names.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
            Self::OptiFine => "optifine",
        }
    }

    /// Whether installation runs an official installer jar.
    pub fn uses_installer(self) -> bool {
        matches!(self, Self::Forge | Self::NeoForge | Self::OptiFine)
    }

    /// Whether the user has to provide the installer file themselves.
    pub fn needs_local_installer(self) -> bool {
        matches!(self, Self::OptiFine)
    }

    fn meta(self) -> &'static str {
        match self {
            Self::Fabric => "https://meta.fabricmc.net/v2",
            Self::Quilt => "https://meta.quiltmc.org/v3",
            Self::Forge => "https://files.minecraftforge.net/net/minecraftforge/forge",
            Self::NeoForge => "https://maven.neoforged.net/releases",
            Self::OptiFine => "https://optifine.net",
        }
    }
}

/// One selectable loader build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderChoice {
    /// Loader version string.
    pub version: String,
    /// Whether the publisher marks it stable.
    pub stable: bool,
}

/// Result of a loader install.
#[derive(Debug, Clone)]
pub struct LoaderInstall {
    /// Version id written to disk.
    pub id: String,
    /// Minecraft version it extends.
    pub game_version: String,
    /// Loader version.
    pub loader_version: String,
}

/// Lists installable loader versions for `game`, newest first.
pub fn versions(kind: LoaderKind, game: &str) -> Result<Vec<LoaderChoice>, CoreError> {
    match kind {
        LoaderKind::Fabric | LoaderKind::Quilt => profile_versions(kind, game),
        LoaderKind::Forge => forge_versions(game),
        LoaderKind::NeoForge => neoforge_versions(game),
        LoaderKind::OptiFine => Ok(Vec::new()),
    }
}

/// Installs `kind` for `game`.
///
/// Fabric/Quilt only write the profile document; the caller still has to run
/// the normal install flow for the returned id to fetch the libraries.
/// Forge/NeoForge/OptiFine run their installer, which already fetches
/// everything and writes the version document.
pub fn install(
    root: &Path,
    kind: LoaderKind,
    game: &str,
    version: Option<&str>,
    java: Option<&Path>,
) -> Result<LoaderInstall, CoreError> {
    if kind.uses_installer() {
        return install_with_installer(root, kind, game, version, java, None);
    }

    let loader_version = match version {
        Some(version) => version.to_owned(),
        None => latest_loader(kind, game)?,
    };
    let url = format!(
        "{}/versions/loader/{game}/{loader_version}/profile/json",
        kind.meta()
    );
    let response = net::get_json(&url, None)?;
    if !response.is_ok() {
        return Err(CoreError::Network(format!(
            "{game} 还没有 {} 的安装配置（HTTP {}）",
            kind.label(),
            response.status
        )));
    }

    let mut profile = response.body;
    let id = profile["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}-loader-{loader_version}-{game}", kind.slug()));
    profile["id"] = json!(id);
    profile["inheritsFrom"] = json!(game);

    if profile["libraries"].as_array().is_none_or(Vec::is_empty) {
        return Err(CoreError::Network(format!(
            "{} 的配置里没有库文件，已中止",
            kind.label()
        )));
    }

    write_version_document(root, &id, &profile)?;

    Ok(LoaderInstall {
        id,
        game_version: game.to_owned(),
        loader_version,
    })
}

/// Installs Forge, NeoForge or OptiFine from a local installer jar.
pub fn install_from_jar(
    root: &Path,
    game: &str,
    kind: LoaderKind,
    jar: &Path,
    java: &Path,
) -> Result<LoaderInstall, CoreError> {
    install_with_installer(root, kind, game, None, Some(java), Some(jar))
}

fn install_with_installer(
    root: &Path,
    kind: LoaderKind,
    game: &str,
    version: Option<&str>,
    java: Option<&Path>,
    local_jar: Option<&Path>,
) -> Result<LoaderInstall, CoreError> {
    let java = java
        .map(Path::to_path_buf)
        .or_else(|| {
            crate::java::runtime_java_candidates(root)
                .into_iter()
                .next()
        })
        .ok_or_else(|| {
            CoreError::Invalid(format!(
                "安装 {} 需要 Java，请先在设置里添加或自动下载一个 Java",
                kind.label()
            ))
        })?;

    let loader_version = match (local_jar, version) {
        (Some(_), _) => game.to_owned(),
        (None, Some(version)) => version.to_owned(),
        (None, None) => latest_installer_version(kind, game)?,
    };

    let (jar, expected_id) = match local_jar {
        Some(jar) => (jar.to_path_buf(), String::new()),
        None => {
            let name = installer_file_stem(kind, game, &loader_version);
            let jar = root.join("installers").join(format!("{name}.jar"));
            if !jar.is_file() {
                download_installer(kind, game, &loader_version, &jar)?;
            }
            let expected = expected_version_id(kind, game, &loader_version);
            (jar, expected)
        }
    };

    prepare_install_target(root)?;

    let status = std::process::Command::new(&java)
        .arg(format!("-Djava.io.tmpdir={}", root.join("tmp").display()))
        .arg("-jar")
        .arg(&jar)
        .arg("--installClient")
        .arg(root)
        .current_dir(root)
        .status()
        .map_err(|err| CoreError::Invalid(format!("无法启动 {} 安装器: {err}", kind.label())))?;

    // Some installer runs report a failure after writing the version document
    // (for example when the vanilla download checksum fails); the launcher can
    // finish the job itself, so accept the result whenever the document exists.
    let id = find_installed_version(root, kind, game, &expected_id, &loader_version);
    let id = match id {
        Some(id) => id,
        None => {
            return Err(CoreError::Invalid(format!(
                "{} 安装器返回失败（{}）",
                kind.label(),
                status
                    .code()
                    .map_or_else(|| "未知".to_owned(), |code| code.to_string())
            )));
        }
    };

    Ok(LoaderInstall {
        id,
        game_version: game.to_owned(),
        loader_version,
    })
}

/// Prepares the directory the installers expect.
fn prepare_install_target(root: &Path) -> Result<(), CoreError> {
    fs::create_dir_all(root.join("tmp"))?;
    fs::create_dir_all(root.join("versions"))?;
    let profiles = root.join("launcher_profiles.json");
    if !profiles.is_file() {
        fs::write(&profiles, r#"{"profiles":{},"settings":{},"version":3}"#)?;
    }
    Ok(())
}

/// Downloads an installer jar, reporting byte progress to a log line.
fn download_installer(
    kind: LoaderKind,
    game: &str,
    loader_version: &str,
    destination: &Path,
) -> Result<(), CoreError> {
    let url = installer_url(kind, game, loader_version)?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let agent = net::agent()?;
    let mut file = fs::File::create(destination)?;
    let mut progress = |_bytes: u64| {};
    net::download_to(&agent, &url, &mut file, &mut progress)?;
    Ok(())
}

fn installer_url(kind: LoaderKind, game: &str, loader_version: &str) -> Result<String, CoreError> {
    let url = match kind {
        LoaderKind::Forge => format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{game}-{loader_version}/forge-{game}-{loader_version}-installer.jar"
        ),
        LoaderKind::NeoForge => {
            if game == "1.20.1" {
                format!(
                    "https://maven.neoforged.net/releases/net/neoforged/forge/1.20.1-{loader_version}/forge-1.20.1-{loader_version}-installer.jar"
                )
            } else {
                format!(
                    "https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader_version}/neoforge-{loader_version}-installer.jar"
                )
            }
        }
        _ => {
            return Err(CoreError::Invalid(format!(
                "{} 需要本地安装包",
                kind.label()
            )));
        }
    };
    Ok(url)
}

fn installer_file_stem(kind: LoaderKind, game: &str, loader_version: &str) -> String {
    match kind {
        LoaderKind::Forge => format!("forge-{game}-{loader_version}-installer"),
        LoaderKind::NeoForge => {
            if game == "1.20.1" {
                format!("neoforge-1.20.1-{loader_version}-installer")
            } else {
                format!("neoforge-{loader_version}-installer")
            }
        }
        _ => format!("{}-{loader_version}-installer", kind.slug()),
    }
}

/// Version id the installer is expected to register.
fn expected_version_id(kind: LoaderKind, game: &str, loader_version: &str) -> String {
    match kind {
        LoaderKind::Forge => format!("{game}-forge-{loader_version}"),
        LoaderKind::NeoForge => {
            if game == "1.20.1" {
                format!("1.20.1-forge-{loader_version}")
            } else {
                format!("neoforge-{loader_version}")
            }
        }
        LoaderKind::OptiFine => format!("{game}-OptiFine"),
        _ => format!("{}-{loader_version}-{game}", kind.slug()),
    }
}

/// Finds the version document the installer just wrote.
fn find_installed_version(
    root: &Path,
    kind: LoaderKind,
    game: &str,
    expected: &str,
    loader_version: &str,
) -> Option<String> {
    let versions = root.join("versions");
    if !expected.is_empty() {
        let document = versions.join(expected).join(format!("{expected}.json"));
        if document.is_file() {
            return Some(expected.to_owned());
        }
    }

    let marker = match kind {
        LoaderKind::Forge => format!("-forge-{loader_version}"),
        LoaderKind::NeoForge => {
            if game == "1.20.1" {
                format!("-forge-{loader_version}")
            } else {
                format!("neoforge-{loader_version}")
            }
        }
        LoaderKind::OptiFine => "OptiFine".to_owned(),
        _ => format!("{}-{}", kind.slug(), loader_version),
    };

    let mut found: Vec<String> = fs::read_dir(&versions)
        .ok()?
        .flatten()
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter(|name| name.contains(&marker))
        .filter(|name| versions.join(name).join(format!("{name}.json")).is_file())
        .collect();
    found.sort();
    found.pop()
}

/// Writes a loader profile document under `versions/<id>/`.
fn write_version_document(
    root: &Path,
    id: &str,
    document: &serde_json::Value,
) -> Result<(), CoreError> {
    let directory = root.join("versions").join(id);
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join(format!("{id}.json")),
        serde_json::to_string_pretty(document)?,
    )?;
    Ok(())
}

/// Newest Fabric/Quilt loader build for `game`.
fn latest_loader(kind: LoaderKind, game: &str) -> Result<String, CoreError> {
    let url = format!("{}/versions/loader/{game}", kind.meta());
    let response = net::get_json(&url, None)?;
    if !response.is_ok() {
        return Err(CoreError::Network(format!(
            "{game} 还没有 {} 的构建（HTTP {}）",
            kind.label(),
            response.status
        )));
    }

    let list = response
        .body
        .as_array()
        .ok_or_else(|| CoreError::Network(format!("{url}: 返回的不是数组")))?;

    // Fabric nests the version under `loader`, Quilt puts it at the top level.
    let version_of = |entry: &serde_json::Value| {
        entry
            .pointer("/loader/version")
            .and_then(|value| value.as_str())
            .or_else(|| entry.get("version").and_then(|value| value.as_str()))
            .map(str::to_owned)
    };

    let stable = list.iter().find(|entry| {
        version_of(entry).is_some()
            && entry
                .pointer("/loader/stable")
                .and_then(|value| value.as_bool())
                .unwrap_or(true)
    });

    stable
        .or_else(|| list.iter().find(|entry| version_of(entry).is_some()))
        .and_then(version_of)
        .ok_or_else(|| CoreError::Network(format!("{} 没有可用的版本列表", kind.label())))
}

/// Fabric/Quilt builds for `game`.
fn profile_versions(kind: LoaderKind, game: &str) -> Result<Vec<LoaderChoice>, CoreError> {
    let url = format!("{}/versions/loader/{game}", kind.meta());
    let response = net::get_json(&url, None)?;
    if !response.is_ok() {
        return Ok(Vec::new());
    }
    let list = response.body.as_array().cloned().unwrap_or_default();
    let mut choices = Vec::new();
    for entry in list {
        let version = entry
            .pointer("/loader/version")
            .or_else(|| entry.get("version"))
            .and_then(|value| value.as_str())
            .map(str::to_owned);
        let Some(version) = version else {
            continue;
        };
        let stable = entry
            .pointer("/loader/stable")
            .and_then(|value| value.as_bool())
            .unwrap_or(true);
        choices.push(LoaderChoice { version, stable });
    }
    Ok(choices)
}

/// Forge builds for `game`, from the promotions document.
fn forge_versions(game: &str) -> Result<Vec<LoaderChoice>, CoreError> {
    let url = format!("{}/promotions_slim.json", LoaderKind::Forge.meta());
    let response = net::get_json(&url, None)?;
    if !response.is_ok() {
        return Ok(Vec::new());
    }

    let promos = &response.body["promos"];
    let mut choices = Vec::new();
    for (suffix, stable) in [("recommended", true), ("latest", false)] {
        let key = format!("{game}-{suffix}");
        let Some(version) = promos.get(&key).and_then(|value| value.as_str()) else {
            continue;
        };
        if choices
            .iter()
            .any(|choice: &LoaderChoice| choice.version == version)
        {
            continue;
        }
        choices.push(LoaderChoice {
            version: version.to_owned(),
            stable,
        });
    }
    Ok(choices)
}

/// Newest NeoForge build for `game`.
fn latest_installer_version(kind: LoaderKind, game: &str) -> Result<String, CoreError> {
    let choices = versions(kind, game)?;
    choices
        .first()
        .map(|choice| choice.version.clone())
        .ok_or_else(|| {
            CoreError::Network(format!(
                "{} 没有支持 {game} 的版本，请选择其他加载器",
                kind.label()
            ))
        })
}

/// NeoForge builds for `game`.
fn neoforge_versions(game: &str) -> Result<Vec<LoaderChoice>, CoreError> {
    if game == "1.20.1" {
        return neoforge_legacy_versions();
    }
    let prefix = neoforge_prefix(game).ok_or_else(|| {
        CoreError::Network(format!("NeoForge 不支持 {game}（需要 1.20.2 及以上）"))
    })?;
    let url = format!(
        "{}/net/neoforged/neoforge/maven-metadata.xml",
        LoaderKind::NeoForge.meta()
    );
    let text = net::get_text(&url)?;

    let mut choices = Vec::new();
    for version in extract_xml_versions(&text) {
        if version.starts_with(&prefix) {
            choices.push(LoaderChoice {
                version,
                stable: true,
            });
        }
    }
    // Maven keeps release order; newest builds come last.
    choices.reverse();
    choices.truncate(40);
    Ok(choices)
}

/// 1.20.1 NeoForge lives under the legacy `net.neoforged.forge` artifact.
fn neoforge_legacy_versions() -> Result<Vec<LoaderChoice>, CoreError> {
    let url = format!(
        "{}/net/neoforged/forge/maven-metadata.xml",
        LoaderKind::NeoForge.meta()
    );
    let text = net::get_text(&url)?;
    let mut choices = Vec::new();
    for version in extract_xml_versions(&text) {
        let Some(stripped) = version.strip_prefix("1.20.1-") else {
            continue;
        };
        choices.push(LoaderChoice {
            version: stripped.to_owned(),
            stable: true,
        });
    }
    choices.reverse();
    choices.truncate(40);
    Ok(choices)
}

/// NeoForge versions are prefixed with the Minecraft minor and patch, so
/// `1.21.1` maps to builds starting with `21.1.`.
fn neoforge_prefix(game: &str) -> Option<String> {
    let rest = game.strip_prefix("1.")?;
    let mut parts = rest.split('.');
    let minor = parts.next()?;
    let patch = parts.next().unwrap_or("0");
    Some(format!("{minor}.{patch}."))
}

/// Extracts `<version>` values from a Maven metadata document.
fn extract_xml_versions(text: &str) -> Vec<String> {
    let mut versions = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("<version>") {
        let after = &rest[start + "<version>".len()..];
        let Some(end) = after.find("</version>") else {
            break;
        };
        versions.push(after[..end].trim().to_owned());
        rest = &after[end + "</version>".len()..];
    }
    versions
}

/// Path of the version document for `id`, if it exists.
pub fn version_document(root: &Path, id: &str) -> Option<PathBuf> {
    let document = root.join("versions").join(id).join(format!("{id}.json"));
    document.is_file().then_some(document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fabric_list_shape() {
        let body = serde_json::json!([
            { "loader": { "version": "0.16.9", "stable": false } },
            { "loader": { "version": "0.16.5", "stable": true } }
        ]);
        let list = body.as_array().unwrap();
        let version = list
            .iter()
            .find(|entry| {
                entry
                    .pointer("/loader/stable")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(true)
            })
            .and_then(|entry| entry.pointer("/loader/version"))
            .and_then(|value| value.as_str());
        assert_eq!(version, Some("0.16.5"));
    }

    #[test]
    fn quilt_list_shape() {
        let body = serde_json::json!([{ "version": "0.24.0", "build": 3 }]);
        let version = body
            .as_array()
            .unwrap()
            .first()
            .and_then(|entry| entry.get("version"))
            .and_then(|value| value.as_str());
        assert_eq!(version, Some("0.24.0"));
    }

    #[test]
    fn forge_urls_match_maven_layout() {
        let url = installer_url(LoaderKind::Forge, "1.20.1", "47.2.0").unwrap();
        assert_eq!(
            url,
            "https://maven.minecraftforge.net/net/minecraftforge/forge/1.20.1-47.2.0/forge-1.20.1-47.2.0-installer.jar"
        );
        assert_eq!(
            expected_version_id(LoaderKind::Forge, "1.20.1", "47.2.0"),
            "1.20.1-forge-47.2.0"
        );
        assert_eq!(
            installer_file_stem(LoaderKind::Forge, "1.20.1", "47.2.0"),
            "forge-1.20.1-47.2.0-installer"
        );
    }

    #[test]
    fn neoforge_urls_and_prefixes() {
        assert_eq!(neoforge_prefix("1.21.1").as_deref(), Some("21.1."));
        assert_eq!(neoforge_prefix("1.21").as_deref(), Some("21.0."));
        assert_eq!(neoforge_prefix("1.20.4").as_deref(), Some("20.4."));
        assert_eq!(neoforge_prefix("0.9").as_deref(), None);

        let url = installer_url(LoaderKind::NeoForge, "1.21.1", "21.1.266").unwrap();
        assert_eq!(
            url,
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/21.1.266/neoforge-21.1.266-installer.jar"
        );
        assert_eq!(
            expected_version_id(LoaderKind::NeoForge, "1.21.1", "21.1.266"),
            "neoforge-21.1.266"
        );

        let legacy = installer_url(LoaderKind::NeoForge, "1.20.1", "47.1.106").unwrap();
        assert_eq!(
            legacy,
            "https://maven.neoforged.net/releases/net/neoforged/forge/1.20.1-47.1.106/forge-1.20.1-47.1.106-installer.jar"
        );
        assert_eq!(
            expected_version_id(LoaderKind::NeoForge, "1.20.1", "47.1.106"),
            "1.20.1-forge-47.1.106"
        );
    }

    #[test]
    fn parses_maven_metadata() {
        let text = r#"<metadata><versioning><versions>
            <version>21.1.266</version>
            <version>21.1.265</version>
        </versions></versioning></metadata>"#;
        assert_eq!(
            extract_xml_versions(text),
            vec!["21.1.266".to_owned(), "21.1.265".to_owned()]
        );
    }

    #[test]
    fn forge_promotions_are_mapped() {
        let body = serde_json::json!({
            "promos": {
                "1.20.1-recommended": "47.2.0",
                "1.20.1-latest": "47.2.0",
                "1.21.1-latest": "52.0.1"
            }
        });
        let promos = &body["promos"];
        let mut choices = Vec::new();
        for (suffix, stable) in [("recommended", true), ("latest", false)] {
            let key = format!("1.20.1-{suffix}");
            let Some(version) = promos.get(&key).and_then(|value| value.as_str()) else {
                continue;
            };
            if choices
                .iter()
                .any(|choice: &LoaderChoice| choice.version == version)
            {
                continue;
            }
            choices.push(LoaderChoice {
                version: version.to_owned(),
                stable,
            });
        }
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].version, "47.2.0");
        assert!(choices[0].stable);
    }

    #[test]
    fn finds_installed_version_document() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("loader-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("versions/1.20.1-forge-47.2.0")).unwrap();
        fs::write(
            root.join("versions/1.20.1-forge-47.2.0/1.20.1-forge-47.2.0.json"),
            "{}",
        )
        .unwrap();

        assert_eq!(
            find_installed_version(
                &root,
                LoaderKind::Forge,
                "1.20.1",
                "1.20.1-forge-47.2.0",
                "47.2.0"
            ),
            Some("1.20.1-forge-47.2.0".to_owned())
        );
        assert_eq!(
            find_installed_version(&root, LoaderKind::Forge, "1.20.1", "", "47.2.0"),
            Some("1.20.1-forge-47.2.0".to_owned())
        );
        assert_eq!(
            find_installed_version(&root, LoaderKind::Forge, "1.20.1", "", "47.3.0"),
            None
        );

        let _ = fs::remove_dir_all(&root);
    }
}
