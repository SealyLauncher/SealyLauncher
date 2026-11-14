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

    /// Discovers a usable root, falling back to a per-user directory.
    ///
    /// macOS runs quarantined apps from a randomised, read-only
    /// `AppTranslocation` path, and `.app` bundles usually live in
    /// `/Applications`; both make `<bundle>/SealyData` impossible to create.
    /// Instead of exiting silently the launcher then keeps its data under the
    /// user's data directory.
    pub fn prepare() -> Result<(Self, bool), CoreError> {
        let exe = std::env::current_exe()
            .map_err(|err| CoreError::RootUnavailable(format!("current_exe: {err}")))?;
        let portable = Self::from_root(root_from_exe(&exe));

        match portable.ensure() {
            Ok(()) => Ok((portable, false)),
            Err(err) => {
                let root = user_data_root()?;
                let fallback = Self::from_root(&root);
                fallback.ensure()?;
                tracing::warn!(
                    error = %err,
                    portable = %portable.root.display(),
                    fallback = %fallback.root.display(),
                    "portable data directory is not writable; using per-user data"
                );
                Ok((fallback, true))
            }
        }
    }

    /// Whether the executable path sits in a macOS App Translocation directory.
    pub fn is_translocated(exe: &Path) -> bool {
        exe.components().any(|component| {
            component
                .as_os_str()
                .to_str()
                .is_some_and(|part| part == "AppTranslocation")
        })
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

    /// Instance book path.
    pub fn instances_file(&self) -> PathBuf {
        self.data.join("instances.json")
    }

    /// Cached authentication tokens.
    pub fn tokens_file(&self) -> PathBuf {
        self.data.join("tokens.json")
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

/// Per-user data root used when the portable directory is read-only.
pub fn user_data_root() -> Result<PathBuf, CoreError> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").ok_or_else(|| {
            CoreError::RootUnavailable(
                "HOME is not set; cannot find Application Support".to_owned(),
            )
        })?;
        return Ok(Path::new(&home)
            .join("Library")
            .join("Application Support")
            .join("SealyLauncher"));
    }

    #[cfg(windows)]
    {
        let base = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .ok_or_else(|| {
                CoreError::RootUnavailable(
                    "APPDATA is not set; cannot find a data directory".to_owned(),
                )
            })?;
        return Ok(Path::new(&base).join("SealyLauncher"));
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(Path::new(&data_home).join("sealy-launcher"));
        }
        let home = std::env::var_os("HOME").ok_or_else(|| {
            CoreError::RootUnavailable("HOME is not set; cannot find a data directory".to_owned())
        })?;
        return Ok(Path::new(&home)
            .join(".local")
            .join("share")
            .join("sealy-launcher"));
    }

    #[allow(unreachable_code)]
    Err(CoreError::RootUnavailable(
        "no per-user data directory for this platform".to_owned(),
    ))
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
