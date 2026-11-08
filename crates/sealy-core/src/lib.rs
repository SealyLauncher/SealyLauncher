//! Core domain logic for SealyLauncher.
//!
//! This crate must stay UI-agnostic: it must never depend on GPUI or any
//! presentation-layer crate.

pub mod config;
pub mod error;
pub mod events;
pub mod install;
pub mod java;
pub mod logging;
pub mod manifest;
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
}

/// Resolves portable paths, ensures directories, and loads settings.
pub fn bootstrap() -> Result<Bootstrap, CoreError> {
    let paths = LauncherPaths::discover()?;
    paths.ensure()?;
    let settings = Settings::load_or_default(&paths.settings_file());
    Ok(Bootstrap { paths, settings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_not_empty() {
        assert!(!version().is_empty());
    }
}
