//! Core domain logic for SealyLauncher.
//!
//! This crate must stay UI-agnostic: it must never depend on GPUI or any
//! presentation-layer crate.

/// Current crate version, sourced from Cargo metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the launcher's user-visible version string.
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_not_empty() {
        assert!(!version().is_empty());
    }
}
