//! Launcher self-update against GitHub releases.
//!
//! The release workflow publishes one archive per platform; the launcher checks
//! the latest release, picks the matching archive and can hand the update to a
//! small script that replaces the running executable after it exits.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CoreError;
use crate::net;

/// GitHub repository the launcher releases from.
pub const REPOSITORY: &str = "SealyLauncher/SealyLauncher";

/// One downloadable release asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetInfo {
    /// File name inside the release.
    pub name: String,
    /// Download URL.
    pub url: String,
    /// Size in bytes, when GitHub reports it.
    pub size: u64,
}

/// A newer release than the running one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseInfo {
    /// Tag name, e.g. `v0.2.0`.
    pub tag: String,
    /// Version without the leading `v`.
    pub version: String,
    /// Release notes (Markdown).
    pub notes: String,
    /// Page to open when the download cannot be automated.
    pub page: String,
    /// Asset matching this platform, when there is one.
    pub asset: Option<AssetInfo>,
}

/// Checks the newest GitHub release and returns it when it is newer.
pub fn check(current: &str) -> Result<Option<ReleaseInfo>, CoreError> {
    let url = format!("https://api.github.com/repos/{REPOSITORY}/releases/latest");
    let response = net::get_json(&url, None)?;
    if !response.is_ok() {
        return Err(CoreError::Network(format!(
            "无法查询更新（HTTP {}）",
            response.status
        )));
    }
    let body = response.body;
    let tag = body["tag_name"].as_str().unwrap_or_default().to_owned();
    if tag.is_empty() {
        return Ok(None);
    }
    let version = tag.trim_start_matches('v').to_owned();
    if !is_newer(current, &version) {
        return Ok(None);
    }

    let assets = body["assets"]
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    Some(AssetInfo {
                        name: entry["name"].as_str()?.to_owned(),
                        url: entry["browser_download_url"].as_str()?.to_owned(),
                        size: entry["size"].as_u64().unwrap_or_default(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(Some(ReleaseInfo {
        tag,
        version,
        notes: body["body"].as_str().unwrap_or_default().to_owned(),
        page: body["html_url"].as_str().unwrap_or_default().to_owned(),
        asset: asset_for_platform(&assets, std::env::consts::OS, std::env::consts::ARCH),
    }))
}

/// Compares two dotted version strings numerically.
pub fn is_newer(current: &str, candidate: &str) -> bool {
    fn parts(value: &str) -> Vec<u64> {
        value
            .trim_start_matches('v')
            .split(['.', '-'])
            .map(|part| part.trim().parse::<u64>().unwrap_or(0))
            .collect()
    }
    let current = parts(current);
    let candidate = parts(candidate);
    let length = current.len().max(candidate.len());
    for index in 0..length {
        let left = current.get(index).copied().unwrap_or(0);
        let right = candidate.get(index).copied().unwrap_or(0);
        if right != left {
            return right > left;
        }
    }
    false
}

/// Picks the release archive for a platform/arch pair.
pub fn asset_for_platform(assets: &[AssetInfo], os: &str, arch: &str) -> Option<AssetInfo> {
    let needle = match (os, arch) {
        ("windows", _) => "windows-x64.zip",
        ("macos", _) => "macos-universal.zip",
        ("linux", _) => "linux-x86_64.tar.zst",
        _ => return None,
    };
    assets
        .iter()
        .find(|asset| asset.name.ends_with(needle))
        .cloned()
}

/// Downloads the asset to `destination`.
pub fn download(asset: &AssetInfo, destination: &Path) -> Result<u64, CoreError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let agent = net::agent()?;
    let mut file = fs::File::create(destination)?;
    let mut progress = |_bytes: u64| {};
    net::download_to(&agent, &asset.url, &mut file, &mut progress)
}

/// Extracts a release archive into `target`, replacing files.
pub fn extract(archive: &Path, target: &Path) -> Result<usize, CoreError> {
    let name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();
    if name.ends_with(".zip") {
        extract_zip(archive, target)
    } else {
        Err(CoreError::Invalid(format!(
            "当前平台暂不支持自动解压 {name}，请手动解压后覆盖安装"
        )))
    }
}

fn extract_zip(archive: &Path, target: &Path) -> Result<usize, CoreError> {
    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|err| CoreError::Invalid(format!("更新包损坏: {err}")))?;
    let mut written = 0;
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|err| CoreError::Invalid(format!("更新包损坏: {err}")))?;
        let Some(path) = entry.enclosed_name() else {
            continue;
        };
        let destination = target.join(path);
        if entry.is_dir() {
            fs::create_dir_all(&destination)?;
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&destination)?;
        std::io::copy(&mut entry, &mut out)?;
        written += 1;
    }
    Ok(written)
}

/// Where the update archive is cached.
pub fn cache_path(data_dir: &Path, asset: &AssetInfo) -> PathBuf {
    data_dir.join("updates").join(&asset.name)
}

/// Windows helper that waits for the launcher to exit, extracts the archive and
/// starts the new executable.
#[cfg(target_os = "windows")]
pub fn apply_windows(archive: &Path, install_dir: &Path, exe: &Path) -> Result<PathBuf, CoreError> {
    let script = install_dir.join("sealy-update.bat");
    let body = format!(
        "@echo off\r\n\
         setlocal\r\n\
         :wait\r\n\
         ping -n 2 127.0.0.1 >nul\r\n\
         del /f /q \"{exe}\" >nul 2>&1\r\n\
         if exist \"{exe}\" goto wait\r\n\
         powershell -NoProfile -Command \"Expand-Archive -LiteralPath '{archive}' -DestinationPath '{dir}' -Force\"\r\n\
         start \"\" \"{exe}\"\r\n\
         del \"%~f0\"\r\n",
        exe = exe.display(),
        archive = archive.display(),
        dir = install_dir.display(),
    );
    fs::write(&script, body)?;
    std::process::Command::new("cmd")
        .args(["/c", "start", "/min", &script.display().to_string()])
        .current_dir(install_dir)
        .spawn()
        .map_err(|err| CoreError::Invalid(format!("无法启动更新脚本: {err}")))?;
    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("0.1.32", "0.1.33"));
        assert!(is_newer("0.1.32", "0.2.0"));
        assert!(!is_newer("0.2.0", "0.1.99"));
        assert!(!is_newer("0.1.32", "0.1.32"));
        assert!(is_newer("v0.1.32", "0.1.40"));
    }

    #[test]
    fn picks_platform_asset() {
        let assets = vec![
            AssetInfo {
                name: "SealyLauncher-0.2.0-windows-x64.zip".to_owned(),
                url: "https://example.invalid/win".to_owned(),
                size: 1,
            },
            AssetInfo {
                name: "SealyLauncher-0.2.0-macos-universal.zip".to_owned(),
                url: "https://example.invalid/mac".to_owned(),
                size: 2,
            },
            AssetInfo {
                name: "SealyLauncher-0.2.0-linux-x86_64.tar.zst".to_owned(),
                url: "https://example.invalid/linux".to_owned(),
                size: 3,
            },
        ];
        assert_eq!(
            asset_for_platform(&assets, "windows", "x86_64")
                .unwrap()
                .url,
            "https://example.invalid/win"
        );
        assert_eq!(
            asset_for_platform(&assets, "linux", "x86_64").unwrap().url,
            "https://example.invalid/linux"
        );
        assert!(asset_for_platform(&assets, "freebsd", "x86_64").is_none());
    }

    #[test]
    fn extracts_release_zip() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("update-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let archive = root.join("release.zip");
        {
            let file = fs::File::create(&archive).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file("SealyLauncher.exe", options).unwrap();
            writer.write_all(b"binary").unwrap();
            writer.start_file("assets/icon.png", options).unwrap();
            writer.write_all(b"png").unwrap();
            writer.finish().unwrap();
        }
        let target = root.join("out");
        let count = extract(&archive, &target).unwrap();
        assert_eq!(count, 2);
        assert_eq!(
            fs::read(target.join("SealyLauncher.exe")).unwrap(),
            b"binary"
        );
        assert!(target.join("assets/icon.png").is_file());
        let _ = fs::remove_dir_all(&root);
    }
    #[test]
    fn caches_asset_under_data_dir() {
        let path = cache_path(
            Path::new("data"),
            &AssetInfo {
                name: "update.zip".to_owned(),
                url: String::new(),
                size: 0,
            },
        );
        assert!(path.ends_with("updates/update.zip"));
    }
}
