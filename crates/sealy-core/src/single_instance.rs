//! Single-instance protection for one portable data directory.

use std::fs::{File, OpenOptions};
use std::path::Path;

use fs2::FileExt;

use crate::CoreError;

/// Acquires an exclusive lock for the portable data directory.
///
/// The returned `File` must be kept alive for the process lifetime.
pub fn acquire(path: &Path) -> Result<File, CoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)?;

    file.try_lock_exclusive()
        .map_err(|err| CoreError::AlreadyRunning(err.to_string()))?;
    Ok(file)
}
