//! Launcher logging bootstrap.

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

use crate::{CoreError, paths::LauncherPaths};

/// Initializes file logging under `SealyData/logs`.
///
/// The returned guard must live for the process lifetime.
pub fn init(paths: &LauncherPaths) -> Result<WorkerGuard, CoreError> {
    paths.ensure()?;
    let appender = tracing_appender::rolling::daily(&paths.logs, "launcher.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .try_init()
        .map_err(|err| CoreError::Logging(err.to_string()))?;

    tracing::info!(
        root = %paths.root.display(),
        data = %paths.data.display(),
        "SealyLauncher logging initialized"
    );
    Ok(guard)
}
