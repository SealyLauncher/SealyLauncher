//! Launcher logging bootstrap.
//!
//! Logs always go to `SealyData/logs`; the settings page can raise the level
//! from `info` to `debug` at runtime so users can hand over a verbose file.

use std::sync::OnceLock;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::reload;

use crate::{CoreError, paths::LauncherPaths};

type FilterHandle = reload::Handle<EnvFilter, tracing_subscriber::Registry>;

/// Live filter handle, set once during [`init`].
static FILTER: OnceLock<FilterHandle> = OnceLock::new();

/// Initializes file logging under `SealyData/logs`.
///
/// The returned guard must live for the process lifetime.
pub fn init(paths: &LauncherPaths, verbose: bool) -> Result<WorkerGuard, CoreError> {
    paths.ensure()?;
    let appender = tracing_appender::rolling::daily(&paths.logs, "launcher.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);

    let (filter, handle) = reload::Layer::new(filter_for(verbose));
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(writer)
                .with_ansi(false),
        )
        .try_init()
        .map_err(|err| CoreError::Logging(err.to_string()))?;

    let _ = FILTER.set(handle);

    tracing::info!(
        root = %paths.root.display(),
        data = %paths.data.display(),
        logs = %paths.logs.display(),
        verbose,
        "SealyLauncher logging initialized"
    );
    Ok(guard)
}

/// Switches between `info` and `debug` without restarting the process.
pub fn set_verbose(verbose: bool) {
    let Some(handle) = FILTER.get() else {
        return;
    };
    if handle.reload(filter_for(verbose)).is_ok() {
        tracing::info!(verbose, "log level changed");
    }
}

/// Builds the filter, honouring `RUST_LOG` when the user set one.
fn filter_for(verbose: bool) -> EnvFilter {
    if let Ok(filter) = EnvFilter::try_from_default_env() {
        return filter;
    }
    EnvFilter::new(if verbose { "debug" } else { "info" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbose_switches_level() {
        let quiet = format!("{}", filter_for(false));
        let loud = format!("{}", filter_for(true));
        assert!(quiet.contains("info") || quiet.is_empty());
        assert!(loud.contains("debug") || loud.is_empty());
    }
}
