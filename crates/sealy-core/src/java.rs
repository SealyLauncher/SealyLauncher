//! Java discovery and Mojang runtime installation.
//!
//! Minecraft versions declare the major version they need (`javaVersion`);
//! Mojang publishes matching runtimes that the launcher may download, so a
//! fresh portable copy can run without a system JDK.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::install::{self, CancelFlag, ProgressHandle};
use crate::net;
use crate::version::Task;

/// Endpoint listing every published runtime.
const RUNTIME_INDEX: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// A usable Java installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaInstall {
    /// Path to the `java` executable.
    pub path: PathBuf,
    /// Reported version, e.g. `21.0.5`.
    pub version: String,
    /// Major version used to match a Minecraft version.
    pub major: u32,
}

impl JavaInstall {
    /// Whether this runtime satisfies `required`.
    pub fn satisfies(&self, required: u32) -> bool {
        required == 0 || self.major == required
    }
}

/// One runtime published for this platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEntry {
    /// Component id such as `java-runtime-delta`.
    pub component: String,
    /// Runtime version, e.g. `21.0.5`.
    pub version: String,
    /// URL of the file manifest.
    pub manifest_url: String,
    /// Major version parsed from `version`.
    pub major: u32,
}

/// Raw shape of one entry in Mojang's runtime index.
#[derive(Debug, Clone, Deserialize)]
struct RuntimeRelease {
    #[serde(default)]
    manifest: RuntimeManifestRef,
    #[serde(default)]
    version: RuntimeVersion,
}

/// Manifest pointer of a runtime release.
#[derive(Debug, Clone, Default, Deserialize)]
struct RuntimeManifestRef {
    #[serde(default)]
    url: String,
}

/// Version block of a runtime release.
#[derive(Debug, Clone, Default, Deserialize)]
struct RuntimeVersion {
    #[serde(default)]
    name: String,
}

/// Probes one executable via `java -version`.
pub fn probe(executable: &Path) -> Option<JavaInstall> {
    if !executable.is_file() {
        return None;
    }

    let output = Command::new(executable).arg("-version").output().ok()?;
    let text = String::from_utf8_lossy(&output.stderr).to_string()
        + &String::from_utf8_lossy(&output.stdout);

    let version = parse_version(&text)?;
    let major = parse_major(&version)?;

    Some(JavaInstall {
        path: executable.to_path_buf(),
        version,
        major,
    })
}

/// Parses `openjdk version "21.0.5" 2024-10-15`.
pub fn parse_version(output: &str) -> Option<String> {
    let marker = "version \"";
    let start = output.find(marker)? + marker.len();
    let rest = &output[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

/// Derives the major version from a full version string.
pub fn parse_major(version: &str) -> Option<u32> {
    let first = version.split(['.', '-', '+']).next()?;
    let major: u32 = first.parse().ok()?;
    // Legacy numbering: 1.8.0 -> 8.
    if major == 1 {
        version
            .split('.')
            .nth(1)
            .and_then(|part| part.parse().ok())
            .or(Some(8))
    } else {
        Some(major)
    }
}

/// Finds Java candidates: launcher runtimes, `JAVA_HOME`, `PATH`, common dirs.
pub fn detect(game_dir: &Path) -> Vec<JavaInstall> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. Runtimes the launcher downloaded into <.minecraft>/runtime/<component>.
    let runtime_dir = game_dir.join("runtime");
    if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            candidates.extend(runtime_java_candidates(&entry.path()));
        }
    }

    // 2. JAVA_HOME.
    if let Ok(home) = std::env::var("JAVA_HOME") {
        candidates.push(Path::new(&home).join("bin").join(java_exe()));
    }

    // 3. PATH.
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            candidates.push(dir.join(java_exe()));
        }
    }

    // 4. Well-known install roots on Windows.
    for root in known_roots() {
        for dir in child_dirs(&root) {
            candidates.push(dir.join("bin").join(java_exe()));
        }
    }

    let mut found: Vec<JavaInstall> = Vec::new();
    for candidate in candidates {
        let Some(install) = probe(&candidate) else {
            continue;
        };
        if found.iter().any(|existing| existing.path == install.path) {
            continue;
        }
        found.push(install);
    }

    found.sort_by_key(|install| std::cmp::Reverse(install.major));
    found
}

/// Name of the Java executable on this platform.
pub fn java_exe() -> &'static str {
    if cfg!(windows) { "java.exe" } else { "java" }
}

/// Every place a Mojang runtime may hide its `java` binary.
///
/// Linux and Windows runtimes expose `bin/java`; macOS ships a `jre.bundle`
/// whose real home is `jre.bundle/Contents/Home/bin/java`.
pub fn runtime_java_candidates(root: &Path) -> Vec<PathBuf> {
    let exe = java_exe();
    [
        root.join("bin").join(exe),
        root.join("jre.bundle")
            .join("Contents")
            .join("Home")
            .join("bin")
            .join(exe),
        root.join("Contents").join("Home").join("bin").join(exe),
    ]
    .into_iter()
    .collect()
}

fn child_dirs(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect()
}

fn known_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(program_files) = std::env::var("ProgramFiles") {
        for vendor in [
            "Java",
            "Eclipse Adoptium",
            "Eclipse Foundation",
            "Microsoft",
            "Zulu",
            "Amazon Corretto",
            "BellSoft",
        ] {
            roots.push(Path::new(&program_files).join(vendor));
        }
    }
    if let Ok(program_files) = std::env::var("ProgramFiles(x86)") {
        roots.push(Path::new(&program_files).join("Java"));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        roots.push(Path::new(&local).join("Programs").join("Eclipse Adoptium"));
    }

    roots
}

/// Platform key used by Mojang's runtime index.
fn platform_key() -> &'static str {
    if cfg!(windows) {
        if cfg!(target_arch = "aarch64") {
            "windows-arm64"
        } else if cfg!(target_arch = "x86") {
            "windows-x86"
        } else {
            "windows-x64"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "mac-os-arm64"
        } else {
            "mac-os"
        }
    } else if cfg!(target_arch = "x86") {
        "linux-i386"
    } else {
        "linux"
    }
}

/// Runtimes Mojang publishes for this platform, newest first.
pub fn fetch_runtimes() -> Result<Vec<RuntimeEntry>, CoreError> {
    let text = net::get_text(RUNTIME_INDEX)?;
    let index: BTreeMap<String, BTreeMap<String, Vec<RuntimeRelease>>> =
        serde_json::from_str(&text)?;

    let Some(components) = index.get(platform_key()) else {
        return Ok(Vec::new());
    };

    let mut entries = Vec::new();
    for (component, releases) in components {
        for release in releases {
            let major = parse_major(&release.version.name).unwrap_or(0);
            entries.push(RuntimeEntry {
                component: component.clone(),
                version: release.version.name.clone(),
                manifest_url: release.manifest.url.clone(),
                major,
            });
        }
    }

    entries.sort_by_key(|entry| std::cmp::Reverse(entry.major));
    Ok(entries)
}

/// Picks the runtime that matches the version's Java requirement.
pub fn pick_runtime<'a>(
    entries: &'a [RuntimeEntry],
    component: &str,
    required: u32,
) -> Option<&'a RuntimeEntry> {
    entries
        .iter()
        .find(|entry| entry.component == component && entry.satisfies(required))
        .or_else(|| {
            entries
                .iter()
                .find(|entry| entry.satisfies(required) && entry.component != "minecraft-java-exe")
        })
        .or_else(|| {
            entries
                .iter()
                .find(|entry| entry.component != "minecraft-java-exe")
        })
}

impl RuntimeEntry {
    fn satisfies(&self, required: u32) -> bool {
        required == 0 || self.major == required
    }
}

/// Ensures a Java runtime matching `required` exists, downloading one if needed.
pub fn ensure(
    game_dir: &Path,
    component: &str,
    required: u32,
    progress: &ProgressHandle,
    cancel: &CancelFlag,
) -> Result<JavaInstall, CoreError> {
    let result = ensure_inner(game_dir, component, required, progress, cancel);

    // The caller polls `finished`; every exit path has to set it.
    let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
    state.finished = true;
    if let Err(err) = &result {
        state.error = Some(err.to_string());
    }

    result
}

fn ensure_inner(
    game_dir: &Path,
    component: &str,
    required: u32,
    progress: &ProgressHandle,
    cancel: &CancelFlag,
) -> Result<JavaInstall, CoreError> {
    if let Some(found) = detect(game_dir)
        .into_iter()
        .find(|install| install.satisfies(required))
    {
        return Ok(found);
    }

    let runtimes = fetch_runtimes()?;
    let entry = pick_runtime(&runtimes, component, required).ok_or_else(|| {
        CoreError::Network(format!("no published Java runtime for version {required}"))
    })?;

    install_runtime(game_dir, entry, progress, cancel)
}

/// Downloads one Mojang runtime into `<game_dir>/runtime/<component>`.
pub fn install_runtime(
    game_dir: &Path,
    entry: &RuntimeEntry,
    progress: &ProgressHandle,
    cancel: &CancelFlag,
) -> Result<JavaInstall, CoreError> {
    #[derive(Deserialize)]
    struct FileIndex {
        files: BTreeMap<String, Entry>,
    }
    #[derive(Deserialize)]
    struct Entry {
        #[serde(rename = "type")]
        kind: String,
        #[serde(default)]
        downloads: Option<Downloads>,
        /// Relative target of a `link` entry.
        #[serde(default)]
        target: Option<String>,
        /// Whether the file must keep its executable bit (Unix runtimes).
        #[serde(default)]
        executable: bool,
    }
    #[derive(Deserialize)]
    struct Downloads {
        #[serde(default)]
        raw: Option<RawDownload>,
    }
    #[derive(Deserialize)]
    struct RawDownload {
        url: String,
        #[serde(default)]
        sha1: String,
        #[serde(default)]
        size: u64,
    }

    let text = net::get_text(&entry.manifest_url)?;
    let index: FileIndex = serde_json::from_str(&text)?;

    let root = game_dir.join("runtime").join(&entry.component);
    std::fs::create_dir_all(&root)?;

    let mut tasks = Vec::new();
    let mut links = Vec::new();
    let mut executables = Vec::new();

    for (relative, file) in index.files {
        match file.kind.as_str() {
            "directory" => {
                let _ = std::fs::create_dir_all(root.join(&relative));
            }
            "link" => {
                if let Some(target) = file.target {
                    links.push((relative, target));
                }
            }
            "file" => {
                let Some(raw) = file.downloads.and_then(|downloads| downloads.raw) else {
                    continue;
                };
                if file.executable {
                    executables.push(relative.clone());
                }
                tasks.push(Task {
                    url: raw.url,
                    path: format!("runtime/{}/{relative}", entry.component),
                    sha1: raw.sha1,
                    size: raw.size,
                });
            }
            _ => {}
        }
    }

    {
        let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
        state.total = tasks.len();
        state.current = format!("java {}", entry.version);
    }

    install::run_tasks(game_dir, tasks, 4, progress, cancel)?;

    create_links(&root, &links);
    mark_executables(&root, &executables);

    let candidates = runtime_java_candidates(&root);
    for candidate in &candidates {
        if let Some(install) = probe(candidate) {
            return Ok(install);
        }
    }

    Err(CoreError::Network(format!(
        "下载的 Java 运行时没有可执行的 {}（检查过：{}）",
        java_exe(),
        candidates
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("、")
    )))
}

/// Recreates the symlinks that Unix runtimes use inside `legal/`.
fn create_links(root: &Path, links: &[(String, String)]) {
    #[cfg(unix)]
    {
        for (relative, target) in links {
            let path = root.join(relative);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::remove_file(&path);
            if let Err(err) = std::os::unix::fs::symlink(target, &path) {
                tracing::debug!(
                    path = %path.display(),
                    error = %err,
                    "could not create runtime symlink"
                );
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = (root, links);
    }
}

/// Restores the executable bit for files the manifest marked as executable.
fn mark_executables(root: &Path, files: &[String]) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for relative in files {
            let path = root.join(relative);
            let Ok(metadata) = std::fs::metadata(&path) else {
                continue;
            };
            let mut permissions = metadata.permissions();
            permissions.set_mode(permissions.mode() | 0o755);
            if let Err(err) = std::fs::set_permissions(&path, permissions) {
                tracing::debug!(
                    path = %path.display(),
                    error = %err,
                    "could not mark runtime file executable"
                );
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = (root, files);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_java_inside_macos_bundle() {
        let root = Path::new("/tmp/runtime/java-runtime-delta");
        let candidates = runtime_java_candidates(root);
        let bundle = format!("jre.bundle/Contents/Home/bin/{}", java_exe());
        assert!(candidates.iter().any(|path| path.ends_with(&bundle)));
        assert!(candidates.iter().any(|path| path.ends_with(java_exe())));
    }

    #[test]
    fn parses_openjdk_banner() {
        let banner = "openjdk version \"21.0.5\" 2024-10-15\nOpenJDK Runtime Environment";
        assert_eq!(parse_version(banner).as_deref(), Some("21.0.5"));
        assert_eq!(parse_major("21.0.5"), Some(21));
        assert_eq!(parse_major("1.8.0_412"), Some(8));
        assert_eq!(parse_major("17"), Some(17));
    }

    #[test]
    fn picks_runtime_for_version() {
        let entries = vec![
            RuntimeEntry {
                component: "java-runtime-alpha".to_owned(),
                version: "16.0.1".to_owned(),
                manifest_url: String::new(),
                major: 16,
            },
            RuntimeEntry {
                component: "java-runtime-delta".to_owned(),
                version: "21.0.5".to_owned(),
                manifest_url: String::new(),
                major: 21,
            },
        ];

        let chosen = pick_runtime(&entries, "java-runtime-delta", 21).unwrap();
        assert_eq!(chosen.version, "21.0.5");
    }

    #[test]
    fn install_satisfies_required_major() {
        let java = JavaInstall {
            path: PathBuf::from("C:/jdk/bin/java.exe"),
            version: "21.0.5".to_owned(),
            major: 21,
        };
        assert!(java.satisfies(21));
        assert!(!java.satisfies(17));
        assert!(java.satisfies(0));
    }
}
