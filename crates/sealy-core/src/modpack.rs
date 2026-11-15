//! Modpack import and export.
//!
//! Supported formats:
//! - Modrinth `.mrpack`: a zip with `modrinth.index.json`, downloaded files and
//!   an `overrides/` (plus `client-overrides/`) tree.
//! - CurseForge `.zip`: a zip with `manifest.json`, an overrides folder and
//!   project/file ids. The ids need the CurseForge API to resolve download
//!   URLs, so those entries are reported as skipped instead of failing the
//!   whole import.
//! - Plain zips with only an overrides tree are rejected: nothing tells the
//!   launcher which Minecraft/loader version they belong to.

use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use sha1::{Digest, Sha1};

use crate::error::CoreError;
use crate::install::{CancelFlag, ProgressHandle};
use crate::loader::LoaderKind;
use crate::net;

/// Modrinth pack index inside the archive.
pub const MRPACK_INDEX: &str = "modrinth.index.json";
/// CurseForge pack manifest inside the archive.
pub const CURSE_MANIFEST: &str = "manifest.json";
/// Prefix of the Modrinth overrides tree.
pub const MRPACK_OVERRIDES: &str = "overrides/";
/// Prefix of the Modrinth client-only overrides tree.
pub const MRPACK_CLIENT_OVERRIDES: &str = "client-overrides/";
/// CurseForge manifest key naming the overrides folder.
const CURSE_OVERRIDES_DEFAULT: &str = "overrides";

/// Where a pack came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackFormat {
    /// Modrinth `.mrpack`.
    Modrinth,
    /// CurseForge zip.
    CurseForge,
}

impl PackFormat {
    /// User-facing name.
    pub fn label(self) -> &'static str {
        match self {
            Self::Modrinth => "Modrinth",
            Self::CurseForge => "CurseForge",
        }
    }
}

/// Loader the pack expects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderSpec {
    /// Loader kind.
    pub kind: LoaderKind,
    /// Specific build, when the pack pins one.
    pub version: Option<String>,
}

/// One file the pack wants downloaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFile {
    /// Path relative to the instance directory.
    pub path: String,
    /// Candidate download URLs, tried in order.
    pub urls: Vec<String>,
    /// Expected SHA-1, when the pack provides one.
    pub sha1: Option<String>,
}

/// Everything the launcher needs to import a pack.
#[derive(Debug, Clone)]
pub struct ModpackPlan {
    /// Pack display name.
    pub name: String,
    /// Pack version string.
    pub version: String,
    /// Minecraft version the pack runs on.
    pub game_version: String,
    /// Loader the pack pins, when it has one.
    pub loader: Option<LoaderSpec>,
    /// Files to download.
    pub files: Vec<PackFile>,
    /// Overrides prefixes inside the archive (with trailing slash).
    pub overrides: Vec<String>,
    /// Entries that cannot be fetched automatically.
    pub skipped: Vec<String>,
    /// Detected format.
    pub format: PackFormat,
}

/// Result of copying a pack into an instance.
#[derive(Debug, Clone, Default)]
pub struct ImportSummary {
    /// Downloaded files.
    pub files: usize,
    /// Extracted override files.
    pub overrides: usize,
    /// Entries skipped because the launcher cannot download them.
    pub skipped: usize,
}

/// Reads the pack metadata without touching the network.
pub fn inspect(archive: &Path) -> Result<ModpackPlan, CoreError> {
    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|err| CoreError::Invalid(format!("不是有效的整合包压缩文件: {err}")))?;

    if zip.by_name(MRPACK_INDEX).is_ok() {
        let index = read_json(&mut zip, MRPACK_INDEX)?;
        return parse_mrpack(archive, &index);
    }
    if zip.by_name(CURSE_MANIFEST).is_ok() {
        let manifest = read_json(&mut zip, CURSE_MANIFEST)?;
        return parse_curse(archive, &manifest);
    }

    Err(CoreError::Invalid(
        "压缩包里没有 modrinth.index.json 或 manifest.json，无法判断整合包信息".to_owned(),
    ))
}

fn read_json(
    zip: &mut zip::ZipArchive<fs::File>,
    name: &str,
) -> Result<serde_json::Value, CoreError> {
    let mut entry = zip.by_name(name).map_err(zip_err)?;
    let mut text = String::new();
    entry.read_to_string(&mut text)?;
    serde_json::from_str(&text).map_err(CoreError::from)
}

fn parse_mrpack(archive: &Path, index: &serde_json::Value) -> Result<ModpackPlan, CoreError> {
    let name = index["name"]
        .as_str()
        .filter(|name| !name.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| archive_stem(archive));
    let version = index["versionId"].as_str().unwrap_or("1.0").to_owned();

    let dependencies = &index["dependencies"];
    let game_version = dependencies["minecraft"]
        .as_str()
        .ok_or_else(|| CoreError::Invalid("整合包没有声明 Minecraft 版本".to_owned()))?
        .to_owned();
    let loader = mrpack_loader(dependencies);

    let mut files = Vec::new();
    let mut skipped = Vec::new();
    if let Some(entries) = index["files"].as_array() {
        for entry in entries {
            let path = entry["path"].as_str().unwrap_or_default().to_owned();
            if path.is_empty() || safe_relative(&path).is_none() {
                skipped.push(format!("不安全的路径: {path}"));
                continue;
            }
            let optional = entry
                .pointer("/env/client")
                .and_then(|value| value.as_str())
                .is_some_and(|env| env == "unsupported" || env == "optional");
            if optional {
                skipped.push(path);
                continue;
            }
            let urls = entry["downloads"]
                .as_array()
                .map(|urls| {
                    urls.iter()
                        .filter_map(|url| url.as_str().map(str::to_owned))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if urls.is_empty() {
                skipped.push(path);
                continue;
            }
            let sha1 = entry
                .pointer("/hashes/sha1")
                .and_then(|value| value.as_str())
                .map(str::to_owned);
            files.push(PackFile { path, urls, sha1 });
        }
    }

    Ok(ModpackPlan {
        name,
        version,
        game_version,
        loader,
        files,
        overrides: vec![
            MRPACK_OVERRIDES.to_owned(),
            MRPACK_CLIENT_OVERRIDES.to_owned(),
        ],
        skipped,
        format: PackFormat::Modrinth,
    })
}

fn mrpack_loader(dependencies: &serde_json::Value) -> Option<LoaderSpec> {
    let mappings = [
        ("fabric-loader", LoaderKind::Fabric),
        ("quilt-loader", LoaderKind::Quilt),
        ("forge", LoaderKind::Forge),
        ("neoforge", LoaderKind::NeoForge),
        ("optifine", LoaderKind::OptiFine),
    ];
    for (key, kind) in mappings {
        if let Some(version) = dependencies.get(key).and_then(|value| value.as_str()) {
            return Some(LoaderSpec {
                kind,
                version: Some(version.to_owned()),
            });
        }
    }
    None
}

fn parse_curse(archive: &Path, manifest: &serde_json::Value) -> Result<ModpackPlan, CoreError> {
    let name = manifest["name"]
        .as_str()
        .filter(|name| !name.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| archive_stem(archive));
    let version = manifest["version"].as_str().unwrap_or("1.0").to_owned();
    let game_version = manifest
        .pointer("/minecraft/version")
        .and_then(|value| value.as_str())
        .ok_or_else(|| CoreError::Invalid("整合包没有声明 Minecraft 版本".to_owned()))?
        .to_owned();

    // `modLoaders[0].id` looks like `forge-47.2.0` or `fabric-0.15.0`.
    let loader = manifest
        .pointer("/minecraft/modLoaders/0/id")
        .and_then(|value| value.as_str())
        .and_then(parse_curse_loader);

    let overrides = manifest["overrides"]
        .as_str()
        .unwrap_or(CURSE_OVERRIDES_DEFAULT)
        .trim_matches('/')
        .to_owned();

    let mut skipped = Vec::new();
    if let Some(entries) = manifest["files"].as_array() {
        for entry in entries {
            let project = entry["projectID"].as_i64().unwrap_or_default();
            let file = entry["fileID"].as_i64().unwrap_or_default();
            skipped.push(format!("CurseForge project {project} / file {file}"));
        }
    }

    Ok(ModpackPlan {
        name,
        version,
        game_version,
        loader,
        files: Vec::new(),
        overrides: if overrides.is_empty() {
            Vec::new()
        } else {
            vec![format!("{overrides}/")]
        },
        skipped,
        format: PackFormat::CurseForge,
    })
}

fn parse_curse_loader(id: &str) -> Option<LoaderSpec> {
    let (name, version) = id.split_once('-')?;
    let kind = match name {
        "forge" => LoaderKind::Forge,
        "neoforge" => LoaderKind::NeoForge,
        "fabric" => LoaderKind::Fabric,
        "quilt" => LoaderKind::Quilt,
        _ => return None,
    };
    Some(LoaderSpec {
        kind,
        version: Some(version.to_owned()),
    })
}

fn zip_err(err: zip::result::ZipError) -> CoreError {
    CoreError::Invalid(format!("整合包压缩文件错误: {err}"))
}

fn archive_stem(archive: &Path) -> String {
    archive
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("整合包")
        .to_owned()
}

/// Normalises a pack-relative path, rejecting anything that escapes the target.
pub fn safe_relative(path: &str) -> Option<PathBuf> {
    let normalised = path.replace('\\', "/");
    let mut result = PathBuf::new();
    for component in Path::new(&normalised).components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }
    (!result.as_os_str().is_empty()).then_some(result)
}

/// Downloads the pack files and extracts its overrides into `instance_dir`.
pub fn import_files(
    instance_dir: &Path,
    archive: &Path,
    plan: &ModpackPlan,
    progress: &ProgressHandle,
    cancel: &CancelFlag,
) -> Result<ImportSummary, CoreError> {
    fs::create_dir_all(instance_dir)?;
    let mut summary = ImportSummary {
        skipped: plan.skipped.len(),
        ..ImportSummary::default()
    };

    {
        let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
        state.total = plan.files.len();
        state.current = "download".to_owned();
    }

    let agent = net::agent()?;
    for entry in &plan.files {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(CoreError::Invalid("已取消".to_owned()));
        }
        let Some(relative) = safe_relative(&entry.path) else {
            summary.skipped += 1;
            continue;
        };
        let destination = instance_dir.join(&relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        if let (Some(expected), true) = (&entry.sha1, destination.is_file())
            && sha1_of(&destination).is_ok_and(|actual| &actual == expected)
        {
            bump(progress, &entry.path);
            summary.files += 1;
            continue;
        }

        let mut downloaded = false;
        for url in &entry.urls {
            {
                let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
                state.current = entry.path.clone();
                state.current_bytes = 0;
            }
            let temp = destination.with_extension("sealy-part");
            let mut file = fs::File::create(&temp)?;
            let mut on_progress = |bytes: u64| {
                let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
                state.current_bytes = bytes;
            };
            let result = net::download_to(&agent, url, &mut file, &mut on_progress);
            drop(file);
            if result.is_err() {
                let _ = fs::remove_file(&temp);
                continue;
            }
            if let Some(expected) = &entry.sha1 {
                let actual = sha1_of(&temp)?;
                if &actual != expected {
                    let _ = fs::remove_file(&temp);
                    continue;
                }
            }
            fs::rename(&temp, &destination)?;
            downloaded = true;
            break;
        }

        if !downloaded {
            return Err(CoreError::Invalid(format!(
                "无法下载 {}，请检查网络后重试",
                entry.path
            )));
        }
        bump(progress, &entry.path);
        summary.files += 1;
    }

    summary.overrides = extract_overrides(instance_dir, archive, &plan.overrides, progress)?;

    let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
    state.finished = true;
    state.current = "done".to_owned();
    Ok(summary)
}

fn bump(progress: &ProgressHandle, path: &str) {
    let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
    state.done += 1;
    state.current = path.to_owned();
}

/// Copies each override tree into the instance directory.
fn extract_overrides(
    instance_dir: &Path,
    archive: &Path,
    prefixes: &[String],
    progress: &ProgressHandle,
) -> Result<usize, CoreError> {
    if prefixes.is_empty() {
        return Ok(0);
    }
    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|err| CoreError::Invalid(format!("无法读取整合包: {err}")))?;
    let mut written = 0;

    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(zip_err)?;
        let Some(entry_path) = entry.enclosed_name() else {
            continue;
        };
        let raw = entry_path.to_string_lossy().replace('\\', "/");
        let Some(prefix) = prefixes
            .iter()
            .find(|prefix| raw.starts_with(prefix.as_str()))
        else {
            continue;
        };
        let relative = &raw[prefix.len()..];
        if relative.is_empty() || relative.ends_with('/') {
            continue;
        }
        let Some(target) = safe_relative(relative) else {
            continue;
        };
        let destination = instance_dir.join(target);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&destination)?;
        std::io::copy(&mut entry, &mut out)?;
        written += 1;
        {
            let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
            state.current = destination
                .strip_prefix(instance_dir)
                .unwrap_or(&destination)
                .display()
                .to_string();
        }
    }

    Ok(written)
}

/// SHA-1 of a file.
pub fn sha1_of(path: &Path) -> Result<String, CoreError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 32 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Folders copied into an exported pack.
const EXPORT_FOLDERS: [&str; 6] = [
    "mods",
    "config",
    "defaultconfigs",
    "resourcepacks",
    "shaderpacks",
    "scripts",
];

/// Exports an instance as a Modrinth-style `.mrpack` zip.
///
/// The pack carries every exported file as an override, so it can be imported
/// again without relying on remote download URLs.
pub fn export(
    instance_dir: &Path,
    destination: &Path,
    name: &str,
    version: &str,
    game_version: &str,
) -> Result<usize, CoreError> {
    let file = fs::File::create(destination)?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let index = serde_json::json!({
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": version,
        "name": name,
        "summary": format!("{name} 实例导出"),
        "files": [],
        "dependencies": { "minecraft": game_version },
    });
    writer.start_file(MRPACK_INDEX, options).map_err(zip_err)?;
    writer.write_all(serde_json::to_string_pretty(&index)?.as_bytes())?;

    let mut count = 0;
    for folder in EXPORT_FOLDERS {
        let root = instance_dir.join(folder);
        if !root.is_dir() {
            continue;
        }
        count += write_tree(
            &mut writer,
            &root,
            &format!("{MRPACK_OVERRIDES}{folder}"),
            options,
        )?;
    }

    writer.finish().map_err(zip_err)?;
    Ok(count)
}

fn write_tree(
    writer: &mut zip::ZipWriter<fs::File>,
    root: &Path,
    prefix: &str,
    options: zip::write::SimpleFileOptions,
) -> Result<usize, CoreError> {
    let mut count = 0;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let target = format!("{prefix}/{name}");
        if path.is_dir() {
            count += write_tree(writer, &path, &target, options)?;
        } else if path.is_file() {
            writer.start_file(target, options).map_err(zip_err)?;
            let mut source = fs::File::open(&path)?;
            std::io::copy(&mut source, writer)?;
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};

    fn scratch(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("modpack-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, data) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap();
    }

    #[test]
    fn inspects_mrpack() {
        let root = scratch("mrpack");
        let index = serde_json::json!({
            "formatVersion": 1,
            "name": "Cool Pack",
            "versionId": "1.4.2",
            "dependencies": { "minecraft": "1.20.1", "fabric-loader": "0.15.0" },
            "files": [
                {
                    "path": "mods/sodium.jar",
                    "hashes": { "sha1": "aa" },
                    "downloads": ["https://example.invalid/sodium.jar"],
                    "env": { "client": "required" }
                },
                {
                    "path": "mods/server-only.jar",
                    "downloads": ["https://example.invalid/x.jar"],
                    "env": { "client": "unsupported" }
                }
            ]
        });
        let archive = root.join("pack.mrpack");
        write_zip(&archive, &[(MRPACK_INDEX, index.to_string().as_bytes())]);

        let plan = inspect(&archive).unwrap();
        assert_eq!(plan.format, PackFormat::Modrinth);
        assert_eq!(plan.name, "Cool Pack");
        assert_eq!(plan.version, "1.4.2");
        assert_eq!(plan.game_version, "1.20.1");
        assert_eq!(
            plan.loader,
            Some(LoaderSpec {
                kind: LoaderKind::Fabric,
                version: Some("0.15.0".to_owned())
            })
        );
        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].path, "mods/sodium.jar");
        assert_eq!(plan.files[0].sha1.as_deref(), Some("aa"));
        assert!(
            plan.skipped
                .iter()
                .any(|entry| entry.contains("server-only"))
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn inspects_curse_manifest() {
        let root = scratch("curse");
        let manifest = serde_json::json!({
            "name": "Forge Pack",
            "version": "2.0.0",
            "minecraft": {
                "version": "1.20.1",
                "modLoaders": [{ "id": "forge-47.2.0", "primary": true }]
            },
            "overrides": "overrides",
            "files": [{ "projectID": 238222, "fileID": 4567, "required": true }]
        });
        let archive = root.join("pack.zip");
        write_zip(
            &archive,
            &[(CURSE_MANIFEST, manifest.to_string().as_bytes())],
        );

        let plan = inspect(&archive).unwrap();
        assert_eq!(plan.format, PackFormat::CurseForge);
        assert_eq!(plan.game_version, "1.20.1");
        assert_eq!(
            plan.loader,
            Some(LoaderSpec {
                kind: LoaderKind::Forge,
                version: Some("47.2.0".to_owned())
            })
        );
        assert!(plan.files.is_empty());
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.overrides, vec!["overrides/".to_owned()]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_unknown_and_escaping_packs() {
        assert!(safe_relative("../evil.jar").is_none());
        assert!(safe_relative("C:\\windows\\system32\\evil.dll").is_none());
        assert!(safe_relative("/etc/passwd").is_none());
        assert_eq!(
            safe_relative("mods\\ok.jar").unwrap(),
            PathBuf::from("mods/ok.jar")
        );

        let root = scratch("unknown");
        let archive = root.join("plain.zip");
        write_zip(&archive, &[("overrides/mods/a.jar", b"x")]);
        assert!(inspect(&archive).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn exports_and_reimports_overrides() {
        let root = scratch("export");
        let instance = root.join("instance");
        fs::create_dir_all(instance.join("mods")).unwrap();
        fs::write(instance.join("mods/a.jar"), b"jar").unwrap();
        fs::create_dir_all(instance.join("config")).unwrap();
        fs::write(instance.join("config/options.txt"), b"fov:90").unwrap();

        let archive = root.join("out.mrpack");
        let count = export(&instance, &archive, "Exported", "1.0", "1.20.1").unwrap();
        assert_eq!(count, 2);

        let plan = inspect(&archive).unwrap();
        assert_eq!(plan.name, "Exported");
        assert_eq!(plan.game_version, "1.20.1");
        assert!(plan.files.is_empty());

        let target = root.join("imported");
        let progress = Arc::new(Mutex::new(crate::install::Progress::default()));
        let cancel = Arc::new(AtomicBool::new(false));
        let summary = import_files(&target, &archive, &plan, &progress, &cancel).unwrap();
        assert_eq!(summary.overrides, 2);
        assert_eq!(fs::read(target.join("mods/a.jar")).unwrap(), b"jar");
        assert_eq!(
            fs::read_to_string(target.join("config/options.txt")).unwrap(),
            "fov:90"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
