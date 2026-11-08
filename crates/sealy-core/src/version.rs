//! Per-version metadata: the document behind `<id>.json`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// `versions/<id>/<id>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDocument {
    /// Version id.
    pub id: String,
    /// Entry point class.
    #[serde(rename = "mainClass")]
    pub main_class: String,
    /// Java version hint, when present.
    #[serde(rename = "javaVersion", default)]
    pub java_version: Option<JavaVersion>,
    /// Game download (the client jar).
    pub downloads: VersionDownloads,
    /// Libraries to put on the classpath.
    #[serde(default)]
    pub libraries: Vec<Library>,
    /// Asset index reference.
    #[serde(rename = "assetIndex", default)]
    pub asset_index: Option<AssetIndexRef>,
    /// Modern argument blocks; not consumed yet.
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

        if let Some(client) = &self.downloads.client {
            tasks.push(Task {
                url: client.url.clone(),
                path: format!("versions/{id}/{id}.jar"),
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
        entries.push(format!("versions/{id}/{id}.jar"));
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
}
