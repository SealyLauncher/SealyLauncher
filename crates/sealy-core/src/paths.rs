//! Portable path discovery.
//!
//! All launcher state lives under `<root>/SealyData`; the game files live in the
//! familiar `<root>/.minecraft` so worlds and mods can be shared with other
//! launchers. On macOS, `<root>` is the directory containing the `.app` bundle,
//! not the bundle's internal binary directory.

use std::path::{Path, PathBuf};

use crate::CoreError;

/// Resolved directories for one portable launcher copy.
#[derive(Debug, Clone)]
pub struct LauncherPaths {
    /// Directory containing the executable (or the `.app` bundle on macOS).
    pub root: PathBuf,
    /// Portable data directory.
    pub data: PathBuf,
    /// Game directory: `<root>/.minecraft`.
    pub game: PathBuf,
    /// Per-instance game directories.
    pub instances: PathBuf,
    /// Download and metadata cache.
    pub cache: PathBuf,
    /// Launcher and game logs.
    pub logs: PathBuf,
    /// Launcher-managed Java runtimes.
    pub java: PathBuf,
    /// OEM cache and downloaded assets.
    pub oem: PathBuf,
}

impl LauncherPaths {
    /// Builds paths rooted at `root`.
    pub fn from_root(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let data = root.join("SealyData");
        Self {
            game: root.join(".minecraft"),
            instances: data.join("instances"),
            cache: data.join("cache"),
            logs: data.join("logs"),
            java: data.join("java"),
            oem: data.join("oem"),
            data,
            root,
        }
    }

    /// Discovers the portable root from the current executable.
    pub fn discover() -> Result<Self, CoreError> {
        let exe = std::env::current_exe()
            .map_err(|err| CoreError::RootUnavailable(format!("current_exe: {err}")))?;
        Ok(Self::from_root(root_from_exe(&exe)))
    }

    /// Creates all directories required for the launcher shell.
    pub fn ensure(&self) -> Result<(), CoreError> {
        for dir in [
            &self.data,
            &self.game,
            &self.instances,
            &self.cache,
            &self.logs,
            &self.java,
            &self.oem,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    /// Settings file path.
    pub fn settings_file(&self) -> PathBuf {
        self.data.join("settings.json")
    }

    /// Account metadata file path.
    pub fn accounts_file(&self) -> PathBuf {
        self.data.join("accounts.json")
    }

    /// UI/window state file path.
    pub fn state_file(&self) -> PathBuf {
        self.data.join("state.json")
    }

    /// Optional OEM configuration path.
    pub fn oem_conf(&self) -> PathBuf {
        self.root.join("OEM.conf")
    }

    /// Single-instance lock path.
    pub fn lock_file(&self) -> PathBuf {
        self.data.join(".lock")
    }
}

/// Returns the launcher root for an executable path.
///
/// If an ancestor ends with `.app`, its parent is used; otherwise the
/// executable's parent directory is used.
pub fn root_from_exe(exe: &Path) -> PathBuf {
    for ancestor in exe.ancestors() {
        if ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".app"))
        {
            return ancestor
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| ancestor.to_path_buf());
        }
    }

    exe.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_executable_uses_parent_directory() {
        let exe = Path::new("C:/Games/SealyLauncher/sealy-launcher.exe");
        assert_eq!(root_from_exe(exe), Path::new("C:/Games/SealyLauncher"));
    }

    #[test]
    fn macos_app_bundle_uses_bundle_parent() {
        let exe = Path::new("C:/Games/SealyLauncher.app/Contents/MacOS/sealy-launcher");
        assert_eq!(root_from_exe(exe), Path::new("C:/Games"));
    }

    #[test]
    fn data_layout_is_portable() {
        let paths = LauncherPaths::from_root("C:/Games/SealyLauncher");
        assert_eq!(paths.data, Path::new("C:/Games/SealyLauncher/SealyData"));
        assert_eq!(paths.game, Path::new("C:/Games/SealyLauncher/.minecraft"));
        assert_eq!(paths.lock_file(), paths.data.join(".lock"));
    }
}
