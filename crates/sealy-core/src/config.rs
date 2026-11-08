//! Persisted launcher settings.
//!
//! Phase 1 intentionally keeps settings in a small JSON file. The schema is
//! versioned so later phases can migrate without silently losing user data.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{CoreError, paths::LauncherPaths};

/// UI language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    /// Simplified Chinese.
    #[default]
    #[serde(rename = "zh-CN")]
    ZhCn,
    /// English.
    #[serde(rename = "en-US")]
    EnUs,
}

/// Theme selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    /// Dark "deep sea" theme.
    #[default]
    #[serde(rename = "dark")]
    Dark,
    /// Reserved for a later light theme.
    #[serde(rename = "light")]
    Light,
}

/// Download source policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SourcePolicy {
    /// Detect region and choose a sensible order.
    #[default]
    #[serde(rename = "auto")]
    Auto,
    /// Official sources only.
    #[serde(rename = "official")]
    Official,
    /// BMCLAPI first, official fallback.
    #[serde(rename = "bmclapi")]
    Bmclapi,
}

/// Reduced-motion preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MotionSetting {
    /// Follow the operating system.
    #[default]
    #[serde(rename = "system")]
    System,
    /// Always animate.
    #[serde(rename = "always")]
    Always,
    /// Never animate.
    #[serde(rename = "never")]
    Never,
}

/// Appearance settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppearanceSettings {
    /// Theme mode.
    pub theme: ThemeMode,
    /// Whether the home background may rotate through imagery.
    pub background_slideshow: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            background_slideshow: true,
        }
    }
}

/// Download settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadSettings {
    /// Version metadata source.
    pub metadata_source: SourcePolicy,
    /// File source.
    pub file_source: SourcePolicy,
    /// BMCLAPI root.
    pub bmclapi_root: String,
    /// Maximum concurrent downloads.
    pub concurrency: u8,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            metadata_source: SourcePolicy::Auto,
            file_source: SourcePolicy::Auto,
            bmclapi_root: "https://bmclapi2.bangbang93.com".to_owned(),
            concurrency: 8,
        }
    }
}

/// Memory bounds in MiB.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySettings {
    /// Minimum heap.
    pub min_mb: u32,
    /// Maximum heap.
    pub max_mb: u32,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            min_mb: 512,
            max_mb: 4096,
        }
    }
}

/// Java settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaSettings {
    /// Allow downloading Mojang Java runtimes.
    pub auto_download: bool,
    /// Optional user-selected Java binary.
    pub default_path: Option<String>,
    /// Default memory bounds.
    pub memory: MemorySettings,
}

impl Default for JavaSettings {
    fn default() -> Self {
        Self {
            auto_download: true,
            default_path: None,
            memory: MemorySettings::default(),
        }
    }
}

/// Launch defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchSettings {
    /// Launch fullscreen.
    pub fullscreen: bool,
    /// Default width.
    pub width: u32,
    /// Default height.
    pub height: u32,
}

impl Default for LaunchSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            width: 854,
            height: 480,
        }
    }
}

/// Persisted launcher settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Schema version.
    pub schema: u32,
    /// UI language.
    pub language: Language,
    /// Appearance.
    pub appearance: AppearanceSettings,
    /// Downloads.
    pub download: DownloadSettings,
    /// Java.
    pub java: JavaSettings,
    /// Launch behavior.
    pub launch: LaunchSettings,
    /// Motion preference.
    pub motion: MotionSetting,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema: 1,
            language: Language::default(),
            appearance: AppearanceSettings::default(),
            download: DownloadSettings::default(),
            java: JavaSettings::default(),
            launch: LaunchSettings::default(),
            motion: MotionSetting::default(),
        }
    }
}

impl Settings {
    /// Loads settings, returning defaults when the file is missing or invalid.
    pub fn load_or_default(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(settings) => settings,
                Err(err) => {
                    tracing::warn!(path = %path.display(), error = %err, "invalid settings; using defaults");
                    Self::default()
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(err) => {
                tracing::warn!(path = %path.display(), error = %err, "cannot read settings; using defaults");
                Self::default()
            }
        }
    }
}

/// Saves settings using a temporary file and a rename.
pub fn save_settings(paths: &LauncherPaths, settings: &Settings) -> Result<(), CoreError> {
    paths.ensure()?;
    let path = paths.settings_file();
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(settings)?;
    std::fs::write(&tmp, text)?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir() -> tempfile::TempDir {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("test-tmp");
        std::fs::create_dir_all(&base).expect("test temp root");
        tempfile::tempdir_in(base).expect("tempdir")
    }

    #[test]
    fn default_settings_are_stable() {
        let settings = Settings::default();
        assert_eq!(settings.schema, 1);
        assert_eq!(settings.language, Language::ZhCn);
        assert_eq!(settings.download.concurrency, 8);
        assert_eq!(settings.java.memory.max_mb, 4096);
    }

    #[test]
    fn settings_roundtrip() {
        let settings = Settings::default();
        let text = serde_json::to_string(&settings).expect("serialize");
        let parsed: Settings = serde_json::from_str(&text).expect("deserialize");
        assert_eq!(parsed, settings);
    }

    #[test]
    fn settings_save_and_load() {
        let dir = test_dir();
        let paths = LauncherPaths::from_root(dir.path());
        let settings = Settings {
            language: Language::EnUs,
            ..Settings::default()
        };
        save_settings(&paths, &settings).expect("save");
        let loaded = Settings::load_or_default(&paths.settings_file());
        assert_eq!(loaded.language, Language::EnUs);
    }
}
