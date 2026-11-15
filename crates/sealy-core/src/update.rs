//! Launcher self-update against GitHub releases.
//!
//! The release workflow publishes one archive per platform plus a `SHA256SUMS`
//! file. This module checks the latest release, downloads the archive with
//! progress and cancellation, verifies its SHA-256, stages the extracted files
//! and then hands them to a small script that replaces the running launcher
//! after it exits. A pending marker lets the next start detect an update that
//! never completed.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::CoreError;
use crate::net;

/// GitHub repository the launcher releases from.
pub const REPOSITORY: &str = "SealyLauncher/SealyLauncher";
/// File name of the checksum sheet attached to every release.
pub const CHECKSUMS_NAME: &str = "SHA256SUMS";

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
    /// Checksum sheet covering the assets.
    pub checksums: Option<AssetInfo>,
}

/// File name → lowercase SHA-256 hex digest.
pub type ChecksumSheet = BTreeMap<String, String>;

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
        checksums: assets
            .iter()
            .find(|asset| asset.name == CHECKSUMS_NAME)
            .cloned(),
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

/// Parses a `sha256sum`-style sheet.
///
/// Accepts both `hash  name` and `hash *name`, ignores blank lines and
/// comment lines, and lowercases the digest.
pub fn parse_checksums(text: &str) -> ChecksumSheet {
    let mut sheet = ChecksumSheet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, char::is_whitespace);
        let Some(digest) = parts.next() else {
            continue;
        };
        let Some(rest) = parts.next() else {
            continue;
        };
        let name = rest.trim_start_matches('*').trim();
        if name.is_empty() || digest.len() != 64 {
            continue;
        }
        sheet.insert(name.to_owned(), digest.to_ascii_lowercase());
    }
    sheet
}

/// Digest for one file name, when the sheet knows about it.
pub fn expected_digest(sheet: &ChecksumSheet, name: &str) -> Option<String> {
    sheet.get(name).cloned().or_else(|| {
        sheet
            .iter()
            .find(|(key, _)| key.rsplit('/').next() == Some(name))
            .map(|(_, value)| value.clone())
    })
}

/// Downloads and parses the release checksum sheet.
pub fn fetch_checksums(asset: &AssetInfo) -> Result<ChecksumSheet, CoreError> {
    let text = net::get_text(&asset.url)?;
    Ok(parse_checksums(&text))
}

/// Lowercase SHA-256 of a file.
pub fn sha256_of(path: &Path) -> Result<String, CoreError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Verifies `path` against an expected lowercase or uppercase digest.
pub fn verify_file(path: &Path, expected: &str) -> Result<(), CoreError> {
    let actual = sha256_of(path)?;
    if actual == expected.to_ascii_lowercase() {
        Ok(())
    } else {
        Err(CoreError::Invalid(format!(
            "更新包校验失败：应为 {expected}，实际为 {actual}"
        )))
    }
}

/// Downloads the asset to `destination`, reporting progress and honouring
/// cancellation. A partial file is removed on failure.
pub fn download_with_progress(
    asset: &AssetInfo,
    destination: &Path,
    downloaded: &AtomicU64,
    cancel: &AtomicBool,
) -> Result<u64, CoreError> {
    downloaded.store(0, Ordering::Relaxed);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let agent = net::agent()?;
    let mut file = fs::File::create(destination)?;
    let result = net::download_to_cancellable(
        &agent,
        &asset.url,
        &mut file,
        &mut |bytes| {
            downloaded.store(bytes, Ordering::Relaxed);
        },
        cancel,
    );
    if result.is_err() {
        drop(file);
        let _ = fs::remove_file(destination);
    }
    result
}

/// Where the update archive is cached.
pub fn cache_path(data_dir: &Path, asset: &AssetInfo) -> PathBuf {
    data_dir.join("updates").join(&asset.name)
}

/// Staging directory for one version.
pub fn staging_dir(data_dir: &Path, version: &str) -> PathBuf {
    data_dir.join("updates").join(format!("staging-{version}"))
}

/// Extracts `archive` into a clean staging directory and returns the folder
/// that contains the release payload.
pub fn stage(archive: &Path, target: &Path) -> Result<PathBuf, CoreError> {
    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    fs::create_dir_all(target)?;
    let name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();

    if name.ends_with(".zip") {
        extract_zip(archive, target)?;
        return Ok(target.to_path_buf());
    }
    if name.ends_with(".tar.zst") {
        extract_tar_zst(archive, target)?;
        return Ok(single_child_dir(target).unwrap_or_else(|| target.to_path_buf()));
    }

    Err(CoreError::Invalid(format!("不支持的更新包格式: {name}")))
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

fn extract_tar_zst(archive: &Path, target: &Path) -> Result<(), CoreError> {
    let direct = std::process::Command::new("tar")
        .args(["--zstd", "-xf"])
        .arg(archive)
        .arg("-C")
        .arg(target)
        .status();
    if let Ok(status) = direct
        && status.success()
    {
        return Ok(());
    }

    // Fall back to a `zstd -d | tar -x` pipeline for tars without zstd support.
    let mut decompressor = std::process::Command::new("zstd")
        .args(["-d", "-c"])
        .arg(archive)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|err| {
            CoreError::Invalid(format!(
                "无法解压更新包（缺少 tar/zstd）: {err}。请安装 zstd 后重试"
            ))
        })?;
    let Some(stdout) = decompressor.stdout.take() else {
        return Err(CoreError::Invalid("zstd 输出不可用".to_owned()));
    };
    let tar = std::process::Command::new("tar")
        .args(["-xf", "-", "-C"])
        .arg(target)
        .stdin(std::process::Stdio::from(stdout))
        .status()?;
    let decompressed = decompressor.wait()?;
    if !tar.success() || !decompressed.success() {
        return Err(CoreError::Invalid(
            "更新包解压失败，请手动下载安装".to_owned(),
        ));
    }
    Ok(())
}

fn single_child_dir(dir: &Path) -> Option<PathBuf> {
    let mut entries = fs::read_dir(dir).ok()?.flatten();
    let first = entries.next()?.path();
    if entries.next().is_some() || !first.is_dir() {
        return None;
    }
    Some(first)
}

/// The app bundle containing `exe`, when the launcher runs from a macOS bundle.
pub fn macos_bundle(exe: &Path) -> Option<PathBuf> {
    exe.ancestors()
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))
        .map(Path::to_path_buf)
}

/// Writes the marker that lets the next start detect an unfinished update.
pub fn write_pending(data_dir: &Path, version: &str) -> Result<(), CoreError> {
    let path = pending_file(data_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let pending = PendingUpdate {
        version: version.to_owned(),
    };
    fs::write(path, serde_json::to_vec_pretty(&pending)?)?;
    Ok(())
}

/// Marker for an update whose replacement script has been started.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingUpdate {
    /// Version that was being installed.
    pub version: String,
}

/// Path of the pending marker.
pub fn pending_file(data_dir: &Path) -> PathBuf {
    data_dir.join("updates").join("pending.json")
}

/// Reads and removes the pending marker; returns it when the installed version
/// still does not match, meaning the replacement likely failed.
pub fn reconcile_pending(data_dir: &Path, current: &str) -> Option<String> {
    let path = pending_file(data_dir);
    let pending = fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<PendingUpdate>(&bytes).ok());
    let _ = fs::remove_file(&path);
    clear_update_cache(data_dir);
    match pending {
        Some(pending) if pending.version != current => Some(pending.version),
        _ => None,
    }
}

/// Removes archives and staging directories after an update finished.
pub fn clear_update_cache(data_dir: &Path) {
    let dir = data_dir.join("updates");
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let _ = fs::remove_dir_all(&path);
        } else {
            let _ = fs::remove_file(&path);
        }
    }
}

/// Everything needed to install one downloaded release.
pub struct UpdateRequest<'a> {
    /// Asset being installed.
    pub asset: &'a AssetInfo,
    /// Where the archive is cached.
    pub destination: &'a Path,
    /// Checksum sheet asset, when the release has one.
    pub checksums: Option<&'a AssetInfo>,
    /// Byte counter updated while downloading.
    pub downloaded: &'a AtomicU64,
    /// Cancellation flag.
    pub cancel: &'a AtomicBool,
    /// Portable data directory.
    pub data_dir: &'a Path,
    /// Version being installed.
    pub version: &'a str,
    /// Directory containing the running launcher.
    pub install_dir: &'a Path,
    /// Running executable.
    pub exe: &'a Path,
}
/// Downloads, verifies, stages and applies an update.
///
/// Returns the replacement script path; the caller is expected to exit so the
/// script can swap the launcher. When `apply` fails the pending marker is
/// removed again.
pub fn install_release(request: UpdateRequest<'_>) -> Result<PathBuf, CoreError> {
    let UpdateRequest {
        asset,
        destination,
        checksums,
        downloaded,
        cancel,
        data_dir,
        version,
        install_dir,
        exe,
    } = request;
    let sheet = match checksums {
        Some(checksums) => Some(fetch_checksums(checksums)?),
        None => None,
    };

    download_with_progress(asset, destination, downloaded, cancel)?;

    if let Some(sheet) = &sheet {
        match expected_digest(sheet, &asset.name) {
            Some(digest) => verify_file(destination, &digest)?,
            None => {
                let _ = fs::remove_file(destination);
                return Err(CoreError::Invalid(
                    "校验文件中缺少更新包的 SHA-256".to_owned(),
                ));
            }
        }
    }

    let staged_root = stage(destination, &staging_dir(data_dir, version))?;
    if cancel.load(Ordering::Relaxed) {
        let _ = fs::remove_dir_all(&staged_root);
        return Err(CoreError::Cancelled);
    }
    write_pending(data_dir, version)?;
    match apply(
        install_dir,
        exe,
        &staged_root,
        data_dir,
        std::process::id(),
        version,
        destination,
    ) {
        Ok(script) => Ok(script),
        Err(err) => {
            let _ = fs::remove_file(pending_file(data_dir));
            Err(err)
        }
    }
}

/// Applies a staged update and returns the replacement script.
///
/// The script waits for `pid` to exit, replaces the launcher on disk and
/// restarts it. It logs to `SealyData/logs/update.log`.
pub fn apply(
    install_dir: &Path,
    exe: &Path,
    staged: &Path,
    data_dir: &Path,
    pid: u32,
    version: &str,
    archive: &Path,
) -> Result<PathBuf, CoreError> {
    // install_dir is unused on macOS, where the bundle itself is replaced.
    let _ = install_dir;

    #[cfg(target_os = "windows")]
    return apply_windows(install_dir, exe, staged, data_dir, pid, version, archive);

    #[cfg(target_os = "macos")]
    return apply_macos(exe, staged, data_dir, pid, version, archive);

    #[cfg(target_os = "linux")]
    return apply_linux(install_dir, exe, staged, data_dir, pid, version, archive);

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = (install_dir, exe, staged, data_dir, pid, version, archive);
        Err(CoreError::Invalid(
            "当前平台暂不支持自动替换，请手动解压覆盖".to_owned(),
        ))
    }
}

fn update_log(data_dir: &Path) -> PathBuf {
    data_dir.join("logs").join("update.log")
}

fn ensure_log(data_dir: &Path) -> Result<PathBuf, CoreError> {
    let log = update_log(data_dir);
    if let Some(parent) = log.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(log)
}

/// Windows: copy the staged files over the install directory after the process
/// exits; restore the previous executable when the copy fails.
#[cfg(target_os = "windows")]
fn apply_windows(
    install_dir: &Path,
    exe: &Path,
    staged: &Path,
    data_dir: &Path,
    pid: u32,
    version: &str,
    archive: &Path,
) -> Result<PathBuf, CoreError> {
    let _ = pid;
    if !staged.join("sealy-launcher.exe").is_file() {
        return Err(CoreError::Invalid(
            "更新包缺少 sealy-launcher.exe".to_owned(),
        ));
    }
    let log = ensure_log(data_dir)?;
    let script = data_dir.join("updates").join("sealy-update.bat");
    fs::create_dir_all(script.parent().unwrap_or(data_dir))?;
    let body = format!(
        "@echo off\r\n\
         setlocal\r\n\
         set \"LOG={log}\"\r\n\
         set /a tries=0\r\n\
         echo [%date% %time%] update to {version} starting >> \"%LOG%\"\r\n\
         :wait\r\n\
         ping -n 2 127.0.0.1 >nul\r\n\
         rem Renaming the running exe fails while it is locked; retry until the\r\n\
         rem launcher has exited, then the name is free for the staged copy.\r\n\
         move /y \"{exe}\" \"{exe}.old\" >nul 2>>\"%LOG%\"\r\n\
         if not errorlevel 1 goto replace\r\n\
         set /a tries+=1\r\n\
         if %tries% LSS 90 goto wait\r\n\
         echo [%date% %time%] update to {version} timed out waiting for exit >> \"%LOG%\"\r\n\
         goto restore\r\n\
         :replace\r\n\
         xcopy \"{staged}\\*\" \"{install}\" /E /Y /I /Q >nul 2>>\"%LOG%\"\r\n\
         if errorlevel 2 goto restore\r\n\
         start \"\" \"{exe}\"\r\n\
         del /f /q \"{exe}.old\" >nul 2>>\"%LOG%\"\r\n\
         del /f /q \"{archive}\" >nul 2>>\"%LOG%\"\r\n\
         rmdir /s /q \"{staged}\" >nul 2>>\"%LOG%\"\r\n\
         echo [%date% %time%] update to {version} finished >> \"%LOG%\"\r\n\
         del \"%~f0\"\r\n\
         exit /b 0\r\n\
         :restore\r\n\
         if exist \"{exe}\" goto relaunch\r\n\
         move /y \"{exe}.old\" \"{exe}\" >nul 2>>\"%LOG%\"\r\n\
         :relaunch\r\n\
         if exist \"{exe}\" start \"\" \"{exe}\"\r\n\
         echo [%date% %time%] update to {version} FAILED, previous files kept >> \"%LOG%\"\r\n\
         del \"%~f0\"\r\n\
         exit /b 1\r\n",
        log = log.display(),
        version = version,
        exe = exe.display(),
        staged = staged.display(),
        install = install_dir.display(),
        archive = archive.display(),
    );
    fs::write(&script, body)?;
    std::process::Command::new("cmd")
        .args(["/c", "start", "/min", &script.display().to_string()])
        .current_dir(install_dir)
        .spawn()
        .map_err(|err| CoreError::Invalid(format!("无法启动更新脚本: {err}")))?;
    Ok(script)
}

/// macOS: replace the `.app` bundle and relaunch it.
#[cfg(target_os = "macos")]
fn apply_macos(
    exe: &Path,
    staged: &Path,
    data_dir: &Path,
    pid: u32,
    version: &str,
    archive: &Path,
) -> Result<PathBuf, CoreError> {
    use std::os::unix::fs::PermissionsExt;

    let bundle =
        macos_bundle(exe).ok_or_else(|| CoreError::Invalid("找不到 .app 包路径".to_owned()))?;
    let staged_app = first_app_bundle(staged)
        .ok_or_else(|| CoreError::Invalid("更新包中缺少 .app".to_owned()))?;
    let log = ensure_log(data_dir)?;
    let script = data_dir.join("updates").join("sealy-update.sh");
    fs::create_dir_all(script.parent().unwrap_or(data_dir))?;
    let bundle_backup = bundle.with_extension("app.old");
    let body = format!(
        "#!/bin/sh\n\
         LOG=\"{log}\"\n\
         echo \"$(date) update to {version} starting\" >> \"$LOG\"\n\
         while kill -0 {pid} 2>/dev/null; do sleep 0.5; done\n\
         sleep 0.5\n\
         rm -rf \"{backup}\"\n\
         if ! mv \"{bundle}\" \"{backup}\"; then echo \"$(date) move bundle failed\" >> \"$LOG\"; exit 1; fi\n\
         if ! cp -R \"{staged_app}\" \"{bundle}\"; then rm -rf \"{bundle}\"; mv \"{backup}\" \"{bundle}\"; echo \"$(date) copy bundle failed\" >> \"$LOG\"; exit 1; fi\n\
         rm -rf \"{backup}\"\n\
         xattr -cr \"{bundle}\" 2>/dev/null\n\
         open \"{bundle}\"\n\
         rm -rf \"{staged}\"\n\
         rm -f \"{archive}\"\n\
         echo \"$(date) update to {version} finished\" >> \"$LOG\"\n\
         rm -f \"$0\"\n",
        log = log.display(),
        version = version,
        pid = pid,
        backup = bundle_backup.display(),
        bundle = bundle.display(),
        staged_app = staged_app.display(),
        staged = staged.display(),
        archive = archive.display(),
    );
    fs::write(&script, body)?;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
    std::process::Command::new("/bin/sh")
        .arg(&script)
        .current_dir(data_dir)
        .spawn()
        .map_err(|err| CoreError::Invalid(format!("无法启动更新脚本: {err}")))?;
    Ok(script)
}

/// macOS: first `.app` directory inside a staged payload.
#[cfg(target_os = "macos")]
fn first_app_bundle(dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        (path.is_dir() && path.extension().is_some_and(|ext| ext == "app")).then_some(path)
    })
}

/// Linux: copy the staged files over the install directory and restart.
#[cfg(target_os = "linux")]
fn apply_linux(
    install_dir: &Path,
    exe: &Path,
    staged: &Path,
    data_dir: &Path,
    pid: u32,
    version: &str,
    archive: &Path,
) -> Result<PathBuf, CoreError> {
    use std::os::unix::fs::PermissionsExt;

    if !staged.join("sealy-launcher").is_file() {
        return Err(CoreError::Invalid("更新包缺少 sealy-launcher".to_owned()));
    }
    let log = ensure_log(data_dir)?;
    let script = data_dir.join("updates").join("sealy-update.sh");
    fs::create_dir_all(script.parent().unwrap_or(data_dir))?;
    let exe_new = exe.with_extension("new");
    let body = format!(
        "#!/bin/sh\n\
         LOG=\"{log}\"\n\
         echo \"$(date) update to {version} starting\" >> \"$LOG\"\n\
         while kill -0 {pid} 2>/dev/null; do sleep 0.5; done\n\
         sleep 0.5\n\
         cp -f \"{staged}/sealy-launcher\" \"{exe_new}\" || {{ echo \"$(date) stage binary failed\" >> \"$LOG\"; exit 1; }}\n\
         chmod +x \"{exe_new}\" || exit 1\n\
         mv -f \"{exe_new}\" \"{exe}\" || {{ echo \"$(date) move binary failed\" >> \"$LOG\"; exit 1; }}\n\
         for f in README.md LICENSE THIRD_PARTY_NOTICES.md OEM.conf.example sealy-launcher.desktop sealy-launcher.png; do\n\
           [ -f \"{staged}/$f\" ] && cp -f \"{staged}/$f\" \"{install}/$f\"\n\
         done\n\
         \"{exe}\" >/dev/null 2>&1 &\n\
         rm -rf \"{staged}\"\n\
         rm -f \"{archive}\"\n\
         echo \"$(date) update to {version} finished\" >> \"$LOG\"\n\
         rm -f \"$0\"\n",
        log = log.display(),
        version = version,
        pid = pid,
        staged = staged.display(),
        exe_new = exe_new.display(),
        exe = exe.display(),
        install = install_dir.display(),
        archive = archive.display(),
    );
    fs::write(&script, body)?;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
    std::process::Command::new("/bin/sh")
        .arg(&script)
        .current_dir(data_dir)
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
    fn parses_checksum_sheet() {
        let text = "\
            ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  SealyLauncher-0.2.0-windows-x64.zip\n\
            0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e *README.md\n\
            # comment\n\
            \n";
        let sheet = parse_checksums(text);
        assert_eq!(
            expected_digest(&sheet, "SealyLauncher-0.2.0-windows-x64.zip").as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
        assert!(expected_digest(&sheet, "README.md").is_some());
        assert!(expected_digest(&sheet, "missing.zip").is_none());
    }

    #[test]
    fn verifies_sha256() {
        let root = temp_root("sha");
        let file = root.join("payload.bin");
        fs::write(&file, b"abc").unwrap();
        assert_eq!(
            sha256_of(&file).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(
            verify_file(
                &file,
                "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
            )
            .is_ok()
        );
        assert!(verify_file(&file, "00").is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stages_zip_payload() {
        let root = temp_root("zip");
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
        let staged = stage(&archive, &root.join("staging")).unwrap();
        assert_eq!(staged, root.join("staging"));
        assert_eq!(
            fs::read(staged.join("SealyLauncher.exe")).unwrap(),
            b"binary"
        );
        assert!(staged.join("assets/icon.png").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reconcile_reports_unfinished_update() {
        let root = temp_root("pending");
        write_pending(&root, "9.9.9").unwrap();
        assert_eq!(reconcile_pending(&root, "0.1.0").as_deref(), Some("9.9.9"));
        assert!(!pending_file(&root).exists());
        assert!(reconcile_pending(&root, "0.1.0").is_none());

        write_pending(&root, "0.1.0").unwrap();
        assert!(reconcile_pending(&root, "0.1.0").is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cleanup_removes_archives_and_staging() {
        let root = temp_root("cleanup");
        let updates = root.join("updates");
        fs::create_dir_all(updates.join("staging-1")).unwrap();
        fs::write(updates.join("release.zip"), b"x").unwrap();
        clear_update_cache(&root);
        assert!(fs::read_dir(&updates).unwrap().next().is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cancelled_download_removes_partial_file() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        use std::time::Duration;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let body = vec![b'x'; 256 * 1024];
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            for chunk in body.chunks(2048) {
                if stream.write_all(chunk).is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        });

        let root = temp_root("cancel");
        let destination = root.join("partial.bin");
        let asset = AssetInfo {
            name: "partial.bin".to_owned(),
            url: format!("http://{address}/partial.bin"),
            size: 256 * 1024,
        };
        let downloaded = Arc::new(AtomicU64::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        let worker = {
            let downloaded = downloaded.clone();
            let cancel = cancel.clone();
            let destination = destination.clone();
            std::thread::spawn(move || {
                download_with_progress(&asset, &destination, &downloaded, &cancel)
            })
        };

        for _ in 0..400 {
            if downloaded.load(Ordering::Relaxed) > 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        cancel.store(true, Ordering::Relaxed);
        let result = worker.join().unwrap();
        assert!(matches!(result, Err(crate::error::CoreError::Cancelled)));
        assert!(!destination.exists(), "partial file must be removed");
        let _ = server.join();
        let _ = fs::remove_dir_all(root);
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

    fn temp_root(tag: &str) -> PathBuf {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("update-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }
}
