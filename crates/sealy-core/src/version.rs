//! Per-version metadata: the document behind `<id>.json`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// `versions/<id>/<id>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDocument {
    /// Version id.
    pub id: String,
    /// Entry point class; empty on profiles that inherit it.
    #[serde(rename = "mainClass", default)]
    pub main_class: String,
    /// Parent version id, for Fabric/Quilt/Forge style profiles.
    #[serde(rename = "inheritsFrom", default)]
    pub inherits_from: Option<String>,
    /// Version id whose client jar should be used, when not this one.
    #[serde(default)]
    pub jar: Option<String>,
    /// `release`, `snapshot`, `old_beta`, …
    #[serde(rename = "type", default)]
    pub version_type: Option<String>,
    /// Java version hint, when present.
    #[serde(rename = "javaVersion", default)]
    pub java_version: Option<JavaVersion>,
    /// Game download (the client jar).
    #[serde(default)]
    pub downloads: VersionDownloads,
    /// Libraries to put on the classpath.
    #[serde(default)]
    pub libraries: Vec<Library>,
    /// Asset index reference.
    #[serde(rename = "assetIndex", default)]
    pub asset_index: Option<AssetIndexRef>,
    /// Modern argument blocks.
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
    /// Id of the version that owns the client jar, filled by [`resolve`].
    #[serde(skip)]
    pub jar_id: Option<String>,
}

/// Java runtime hint shipped with the version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    /// Component id, e.g. `java-runtime-delta`.
    #[serde(default)]
    pub component: String,
    /// Major version.
    #[serde(rename = "majorVersion", default)]
    pub major_version: u32,
}

/// Downloadable parts of the version itself.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionDownloads {
    /// Client jar.
    #[serde(default)]
    pub client: Option<Artifact>,
}

/// Asset index pointer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexRef {
    /// Index id.
    pub id: String,
    /// URL of the index document.
    pub url: String,
    /// Expected SHA-1.
    #[serde(default)]
    pub sha1: String,
    /// Expected size.
    #[serde(default)]
    pub size: u64,
}

/// One library entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    /// Maven-style coordinate.
    pub name: String,
    /// Legacy base URL for directly hosted artifacts.
    #[serde(default)]
    pub url: Option<String>,
    /// Platform rules.
    #[serde(default)]
    pub rules: Vec<Rule>,
    /// Explicit downloads block.
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
}

/// Downloads available for a library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    /// The plain jar.
    #[serde(default)]
    pub artifact: Option<Artifact>,
}

/// A downloadable file with its hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Relative path under the target root.
    #[serde(default)]
    pub path: String,
    /// Expected SHA-1.
    #[serde(default)]
    pub sha1: String,
    /// Expected size in bytes.
    #[serde(default)]
    pub size: u64,
    /// Absolute URL.
    #[serde(default)]
    pub url: String,
}

/// A rule restricting a library to some platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// `allow` or `disallow`.
    pub action: RuleAction,
    /// Optional OS constraint.
    #[serde(default)]
    pub os: Option<OsRule>,
}

/// Rule verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    /// Permit the entry.
    Allow,
    /// Reject the entry.
    Disallow,
}

/// OS constraint of a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    /// `windows`, `osx` or `linux`.
    #[serde(default)]
    pub name: Option<String>,
    /// Architecture, rarely set.
    #[serde(default)]
    pub arch: Option<String>,
}

/// The launcher's own platform id.
pub fn current_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

impl Rule {
    fn matches(&self) -> bool {
        match &self.os {
            None => true,
            Some(os) => {
                let name_ok = os.name.as_deref().is_none_or(|name| name == current_os());
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

impl Library {
    /// Whether this library applies to the current platform.
    pub fn allowed(&self) -> bool {
        if self.rules.is_empty() {
            return true;
        }

        let mut allowed = false;
        for rule in &self.rules {
            if rule.matches() {
                allowed = rule.action == RuleAction::Allow;
            }
        }
        allowed
    }

    /// Concrete artifact for the current platform, if any.
    pub fn artifact(&self) -> Option<Artifact> {
        if let Some(artifact) = self
            .downloads
            .as_ref()
            .and_then(|downloads| downloads.artifact.as_ref())
        {
            return Some(artifact.clone());
        }

        // Legacy entries carry only a coordinate and a base URL.
        let base = self.url.as_deref()?;
        let path = maven_path(&self.name)?;
        Some(Artifact {
            url: format!("{}/{path}", base.trim_end_matches('/')),
            path,
            sha1: String::new(),
            size: 0,
        })
    }
}

/// Turns `group:artifact:version[:classifier]` into a repository path.
pub fn maven_path(coordinate: &str) -> Option<String> {
    let mut parts = coordinate.split(':');
    let group = parts.next()?;
    let artifact = parts.next()?;
    let version = parts.next()?;
    let classifier = parts.next();

    let file = match classifier {
        Some(classifier) => format!("{artifact}-{version}-{classifier}.jar"),
        None => format!("{artifact}-{version}.jar"),
    };
    Some(format!(
        "{}/{artifact}/{version}/{file}",
        group.replace('.', "/")
    ))
}

/// A single file the installer has to fetch.
#[derive(Debug, Clone)]
pub struct Task {
    /// Absolute URL.
    pub url: String,
    /// Destination relative to the game root.
    pub path: String,
    /// Expected SHA-1; empty when unknown.
    pub sha1: String,
    /// Expected size; zero when unknown.
    pub size: u64,
}

impl VersionDocument {
    /// Every file needed to install this version.
    pub fn plan(&self, id: &str) -> Vec<Task> {
        let mut tasks = Vec::new();
        let jar = self.jar_owner(id);

        if let Some(client) = &self.downloads.client {
            tasks.push(Task {
                url: client.url.clone(),
                path: format!("versions/{jar}/{jar}.jar"),
                sha1: client.sha1.clone(),
                size: client.size,
            });
        }

        for library in &self.libraries {
            if !library.allowed() {
                continue;
            }
            if let Some(artifact) = library.artifact() {
                tasks.push(Task {
                    url: artifact.url,
                    path: format!("libraries/{}", artifact.path),
                    sha1: artifact.sha1,
                    size: artifact.size,
                });
            }
        }

        tasks
    }

    /// Version id that owns the client jar.
    pub fn jar_owner(&self, id: &str) -> String {
        self.jar_id.clone().unwrap_or_else(|| id.to_owned())
    }

    /// Main class of the version.
    pub fn main_class(&self) -> &str {
        &self.main_class
    }

    /// Classpath entries, in declaration order.
    pub fn classpath(&self, id: &str) -> Vec<String> {
        let mut entries = Vec::new();
        for library in &self.libraries {
            if !library.allowed() {
                continue;
            }
            if let Some(path) = maven_path(&library.name) {
                entries.push(format!("libraries/{path}"));
            }
        }
        let jar = self.jar_owner(id);
        entries.push(format!("versions/{jar}/{jar}.jar"));
        entries
    }

    /// Group id, artifact id and version of every library, keyed by coordinate.
    pub fn coordinates(&self) -> BTreeMap<String, String> {
        self.libraries
            .iter()
            .map(|library| (library.name.clone(), library.name.clone()))
            .collect()
    }
}

/// Loads a version document from disk.
pub fn load(path: &std::path::Path) -> Result<VersionDocument, CoreError> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

/// Loads `id` and merges every `inheritsFrom` parent into it.
///
/// Fabric/Quilt/Forge profiles only carry their own main class and libraries;
/// the game jar, asset index, Java hint and argument blocks come from the
/// vanilla parent.
pub fn resolve(root: &std::path::Path, id: &str) -> Result<VersionDocument, CoreError> {
    let mut chain: Vec<VersionDocument> = Vec::new();
    let mut current = id.to_owned();
    let mut seen = std::collections::BTreeSet::new();

    loop {
        if !seen.insert(current.clone()) {
            return Err(CoreError::Network(format!(
                "version {id} 的 inheritsFrom 出现循环"
            )));
        }

        let path = root
            .join("versions")
            .join(&current)
            .join(format!("{current}.json"));
        let doc = load(&path)?;
        let parent = doc
            .inherits_from
            .clone()
            .filter(|parent| !parent.is_empty());
        chain.push(doc);

        match parent {
            Some(parent) => current = parent,
            None => break,
        }
    }

    // Parents first, so each child can override.
    chain.reverse();
    let mut jar_id = None;
    let mut merged = chain.remove(0);
    jar_id = jar_owner_of(&merged, jar_id);

    for child in chain {
        jar_id = jar_owner_of(&child, jar_id);
        merged = merge(merged, child);
    }

    merged.jar_id = jar_id;
    Ok(merged)
}

/// Tracks which version supplies the client jar.
fn jar_owner_of(doc: &VersionDocument, current: Option<String>) -> Option<String> {
    let mut owner = current;
    if doc.downloads.client.is_some() {
        owner = Some(doc.id.clone());
    }
    if let Some(jar) = doc.jar.clone().filter(|jar| !jar.is_empty()) {
        owner = Some(jar);
    }
    owner
}

/// Overlays `child` on top of `parent`.
fn merge(parent: VersionDocument, child: VersionDocument) -> VersionDocument {
    let mut libraries = child.libraries.clone();
    let kept: std::collections::BTreeSet<String> = libraries.iter().map(coordinate_key).collect();
    for library in parent.libraries {
        if !kept.contains(&coordinate_key(&library)) {
            libraries.push(library);
        }
    }

    VersionDocument {
        id: child.id,
        main_class: if child.main_class.is_empty() {
            parent.main_class
        } else {
            child.main_class
        },
        inherits_from: None,
        jar: None,
        version_type: child.version_type.or(parent.version_type),
        java_version: child.java_version.or(parent.java_version),
        downloads: if child.downloads.client.is_some() {
            child.downloads
        } else {
            parent.downloads
        },
        libraries,
        asset_index: child.asset_index.or(parent.asset_index),
        arguments: merge_arguments(parent.arguments, child.arguments),
        jar_id: None,
    }
}

/// `group:artifact` — the identity used to deduplicate libraries.
fn coordinate_key(library: &Library) -> String {
    let mut parts = library.name.split(':');
    let group = parts.next().unwrap_or_default();
    let artifact = parts.next().unwrap_or_default();
    format!("{group}:{artifact}")
}

/// Concatenates `game`/`jvm` argument lists, parent first.
fn merge_arguments(
    parent: Option<serde_json::Value>,
    child: Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    match (parent, child) {
        (None, child) => child,
        (parent, None) => parent,
        (Some(parent), Some(child)) => {
            let mut merged = serde_json::Map::new();
            for key in ["game", "jvm"] {
                let mut tokens = Vec::new();
                for value in [&parent, &child] {
                    if let Some(items) = value.get(key).and_then(|value| value.as_array()) {
                        tokens.extend(items.iter().cloned());
                    }
                }
                if !tokens.is_empty() {
                    merged.insert(key.to_owned(), serde_json::Value::Array(tokens));
                }
            }
            Some(serde_json::Value::Object(merged))
        }
    }
}

/// A version currently installed under `<game>/versions`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledVersion {
    /// Version id.
    pub id: String,
    /// Path of `<id>.json`.
    pub document: std::path::PathBuf,
}

/// Scans `<game>/versions` for installed versions, newest first.
pub fn installed(game_dir: &std::path::Path) -> Vec<InstalledVersion> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(game_dir.join("versions")) else {
        return found;
    };

    for entry in entries.flatten() {
        let id = entry.file_name().to_string_lossy().to_string();
        let document = entry.path().join(format!("{id}.json"));
        if document.is_file() {
            found.push(InstalledVersion { id, document });
        }
    }

    found.sort_by(|a, b| b.id.cmp(&a.id));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"{
        "id": "1.21.11",
        "mainClass": "net.minecraft.client.main.Main",
        "downloads": { "client": { "sha1": "aa", "size": 10, "url": "https://x/client.jar" } },
        "libraries": [
            { "name": "com.mojang:logging:1.1.1",
              "downloads": { "artifact": { "path": "com/mojang/logging/1.1.1/logging-1.1.1.jar",
                                           "sha1": "bb", "size": 5, "url": "https://x/log.jar" } } },
            { "name": "org.lwjgl:lwjgl:3.3.3",
              "rules": [{ "action": "allow", "os": { "name": "windows" } }],
              "url": "https://libraries.minecraft.net/" }
        ]
    }"#;

    #[test]
    fn plans_client_and_platform_libraries() {
        let doc: VersionDocument = serde_json::from_str(DOC).unwrap();
        let tasks = doc.plan("1.21.11");
        assert!(
            tasks
                .iter()
                .any(|t| t.path == "versions/1.21.11/1.21.11.jar")
        );
        assert!(tasks.iter().any(|t| t.path.ends_with("logging-1.1.1.jar")));

        let lwjgl: Library = serde_json::from_str(
            r#"{"name":"org.lwjgl:lwjgl:3.3.3","url":"https://libraries.minecraft.net/",
                "rules":[{"action":"allow","os":{"name":"windows"}}]}"#,
        )
        .unwrap();
        assert_eq!(
            lwjgl.artifact().unwrap().path,
            "org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3.jar"
        );
        assert_eq!(lwjgl.allowed(), cfg!(target_os = "windows"));
    }

    #[test]
    fn builds_classpath() {
        let doc: VersionDocument = serde_json::from_str(DOC).unwrap();
        let classpath = doc.classpath("1.21.11");
        assert_eq!(classpath.last().unwrap(), "versions/1.21.11/1.21.11.jar");
    }

    #[test]
    fn resolves_inherited_profile() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("inherit-{stamp}"));

        let write = |id: &str, body: &str| {
            let dir = root.join("versions").join(id);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("{id}.json")), body).unwrap();
        };

        write(
            "vanilla",
            r#"{"id":"vanilla","mainClass":"net.minecraft.client.main.Main",
                "javaVersion":{"component":"java-runtime-delta","majorVersion":21},
                "downloads":{"client":{"sha1":"aa","size":10,"url":"https://x/client.jar"}},
                "assetIndex":{"id":"17","url":"https://x/17.json"},
                "arguments":{"game":["--gameDir","${game_directory}"],"jvm":["-cp","${classpath}"]},
                "libraries":[{"name":"com.mojang:logging:1.1.1"},
                             {"name":"org.lwjgl:lwjgl:3.3.3"}]}"#,
        );
        write(
            "fabric-loader-0.16.5-vanilla",
            r#"{"id":"fabric-loader-0.16.5-vanilla","inheritsFrom":"vanilla",
                "mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
                "arguments":{"game":["--fabric"],"jvm":[]},
                "libraries":[{"name":"net.fabricmc:fabric-loader:0.16.5"},
                             {"name":"com.mojang:logging:2.0.0"}]}"#,
        );

        let doc = resolve(&root, "fabric-loader-0.16.5-vanilla").unwrap();
        assert_eq!(
            doc.main_class,
            "net.fabricmc.loader.impl.launch.knot.KnotClient"
        );
        assert_eq!(doc.jar_owner("fabric-loader-0.16.5-vanilla"), "vanilla");
        assert_eq!(doc.asset_index.as_ref().unwrap().id, "17");
        assert_eq!(doc.java_version.as_ref().unwrap().major_version, 21);
        assert_eq!(doc.libraries[0].name, "net.fabricmc:fabric-loader:0.16.5");
        assert!(
            doc.libraries
                .iter()
                .any(|library| library.name == "com.mojang:logging:2.0.0")
        );
        assert!(
            !doc.libraries
                .iter()
                .any(|library| library.name == "com.mojang:logging:1.1.1"),
            "child overrides parent library"
        );
        assert!(
            doc.libraries
                .iter()
                .any(|library| library.name.contains("lwjgl"))
        );

        let classpath = doc.classpath("fabric-loader-0.16.5-vanilla");
        assert_eq!(classpath.last().unwrap(), "versions/vanilla/vanilla.jar");

        let game = doc.arguments.as_ref().unwrap()["game"].as_array().unwrap();
        assert!(game.iter().any(|item| item == "--fabric"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
