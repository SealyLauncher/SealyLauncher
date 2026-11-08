//! Installing one version: metadata, client jar and libraries.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use sha1::{Digest, Sha1};

use crate::config::SourcePolicy;
use crate::error::CoreError;
use crate::manifest;
use crate::net;
use crate::version::{self, Task};

/// Snapshot of an install, shared between the worker and the UI.
#[derive(Debug, Clone, Default)]
pub struct Progress {
    /// Files planned.
    pub total: usize,
    /// Files finished (including skipped ones).
    pub done: usize,
    /// File currently being fetched.
    pub current: String,
    /// Bytes written so far.
    pub bytes: u64,
    /// Set once the worker stops, successfully or not.
    pub finished: bool,
    /// Failure message.
    pub error: Option<String>,
}

/// Shared handle the worker writes progress into.
pub type ProgressHandle = Arc<Mutex<Progress>>;

/// What an install produced.
#[derive(Debug, Clone)]
pub struct Installed {
    /// Installed version id.
    pub id: String,
    /// Files fetched or verified.
    pub files: usize,
    /// Bytes fetched.
    pub bytes: u64,
}

/// Installs `id` under `root`, fetching metadata first.
pub fn install(
    root: &Path,
    id: &str,
    policy: SourcePolicy,
    progress: ProgressHandle,
) -> Result<Installed, CoreError> {
    let result = install_inner(root, id, policy, &progress);
    let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
    state.finished = true;
    if let Err(err) = &result {
        state.error = Some(err.to_string());
    }
    result
}

fn install_inner(
    root: &Path,
    id: &str,
    policy: SourcePolicy,
    progress: &ProgressHandle,
) -> Result<Installed, CoreError> {
    let manifest = manifest::fetch_manifest(policy)?;
    let summary = manifest
        .versions
        .iter()
        .find(|version| version.id == id)
        .ok_or_else(|| CoreError::Network(format!("version {id} is not in the manifest")))?;

    let body = net::get_text(&summary.url)?;
    let doc: version::VersionDocument = serde_json::from_str(&body)?;

    let version_dir = root.join("versions").join(id);
    fs::create_dir_all(&version_dir)?;
    fs::write(version_dir.join(format!("{id}.json")), &body)?;

    let tasks = doc.plan(id);
    {
        let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
        state.total = tasks.len();
    }

    let bytes = run_tasks(root, tasks, progress)?;

    Ok(Installed {
        id: id.to_owned(),
        files: {
            let state = progress.lock().unwrap_or_else(|err| err.into_inner());
            state.done
        },
        bytes,
    })
}

/// Downloads every task, skipping files whose SHA-1 already matches.
pub fn run_tasks(
    root: &Path,
    tasks: Vec<Task>,
    progress: &ProgressHandle,
) -> Result<u64, CoreError> {
    let agent = net::agent()?;
    let mut total_bytes = 0_u64;

    for task in tasks {
        let destination = root.join(&task.path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        if is_current(&destination, &task) {
            bump(progress, &task.path, 0);
            continue;
        }

        {
            let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
            state.current = task.path.clone();
        }

        let mut file = fs::File::create(&destination)?;
        let written = net::download_to(&agent, &task.url, &mut file)?;
        drop(file);

        if !task.sha1.is_empty() {
            let actual = sha1_of(&destination)?;
            if !actual.eq_ignore_ascii_case(&task.sha1) {
                let _ = fs::remove_file(&destination);
                return Err(CoreError::Network(format!(
                    "checksum mismatch for {}",
                    task.path
                )));
            }
        }

        total_bytes += written;
        bump(progress, &task.path, written);
    }

    Ok(total_bytes)
}

fn bump(progress: &ProgressHandle, path: &str, bytes: u64) {
    let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
    state.done += 1;
    state.current = path.to_owned();
    state.bytes += bytes;
}

fn is_current(destination: &PathBuf, task: &Task) -> bool {
    let Ok(metadata) = fs::metadata(destination) else {
        return false;
    };
    if task.size > 0 && metadata.len() != task.size {
        return false;
    }
    if task.sha1.is_empty() {
        return false;
    }
    match sha1_of(destination) {
        Ok(actual) => actual.eq_ignore_ascii_case(&task.sha1),
        Err(_) => false,
    }
}

/// Hex SHA-1 of a file.
pub fn sha1_of(path: &Path) -> Result<String, CoreError> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scratch directory under `target/`, since the system temp dir is not
    /// always writable inside the build sandbox.
    fn scratch(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/test-scratch")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn hashes_known_content() {
        let dir = scratch("sha1");
        let file = dir.join("a.txt");
        fs::write(&file, b"abc").unwrap();
        assert_eq!(
            sha1_of(&file).unwrap(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    #[test]
    fn skips_matching_files() {
        let dir = scratch("skip");
        let file = dir.join("b.txt");
        fs::write(&file, b"abc").unwrap();
        let task = Task {
            url: "https://example.invalid/b.txt".to_owned(),
            path: "b.txt".to_owned(),
            sha1: "a9993e364706816aba3e25717850c26c9cd0d89d".to_owned(),
            size: 3,
        };
        assert!(is_current(&file, &task));

        let progress = Arc::new(Mutex::new(Progress::default()));
        let bytes = run_tasks(&dir, vec![task], &progress).unwrap();
        assert_eq!(bytes, 0);
        assert_eq!(progress.lock().unwrap().done, 1);
    }
}
