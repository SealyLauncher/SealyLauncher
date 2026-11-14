//! Core domain logic for SealyLauncher.
//!
//! This crate must stay UI-agnostic: it must never depend on GPUI or any
//! presentation-layer crate.

pub mod accounts;
pub mod assets;
pub mod auth;
pub mod authlib;
pub mod config;
pub mod error;
pub mod events;
pub mod install;
pub mod instances;
pub mod java;
pub mod launch;
pub mod loader;
pub mod logging;
pub mod manifest;
pub mod microsoft;
pub mod net;
pub mod paths;
pub mod single_instance;
pub mod version;

pub use config::Settings;
pub use error::CoreError;
pub use paths::LauncherPaths;

/// Current crate version, sourced from Cargo metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the launcher's user-visible version string.
pub fn version() -> &'static str {
    VERSION
}

/// Bootstrap data handed to the UI.
#[derive(Debug, Clone)]
pub struct Bootstrap {
    /// Portable paths for this copy.
    pub paths: LauncherPaths,
    /// Loaded settings.
    pub settings: Settings,
    /// True when the portable directory was read-only and per-user data is in use.
    pub data_fallback: bool,
    /// True when macOS is running the bundle from an App Translocation path.
    pub translocated: bool,
}

/// Resolves usable paths (falling back to per-user data), then loads settings.
pub fn bootstrap() -> Result<Bootstrap, CoreError> {
    let exe = std::env::current_exe().ok();
    let (paths, data_fallback) = LauncherPaths::prepare()?;
    let settings = Settings::load_or_default(&paths.settings_file());
    Ok(Bootstrap {
        paths,
        settings,
        data_fallback,
        translocated: exe.as_deref().is_some_and(LauncherPaths::is_translocated),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_not_empty() {
        assert!(!version().is_empty());
    }
}
